use super::*;

define_rule! {
  /// Highlights recipe parameters that never get read anywhere in the recipe
  /// body (unless `set export` is on).
  UnusedRecipeParameterRule {
    id: "unused-recipe-parameters",
    message: "unused parameter",
    run(context) {
      let default_script = context.setting_enabled("default-script");

      let exported = context.setting_enabled("export");

      let positional_arguments_enabled = context.setting_enabled("positional-arguments");

      let recipes = context.document().recipes();

      recipes
        .iter()
        .filter_map(|recipe| {
          context
            .scope()
            .recipe_identifier_usage
            .get(&recipe.name.value)
            .map(|identifiers| (recipe, identifiers))
        })
        .flat_map(|(recipe, identifiers)| {
          let recipe_enables_positional_arguments =
            positional_arguments_enabled || recipe.has_attribute("positional-arguments");

          let (positional_usage, uses_all) = if recipe_enables_positional_arguments {
            let (indices, uses_all) = UnusedRecipeParameterRule::positional_argument_usage(recipe);

            (indices, recipe.runs_as_script(default_script) || uses_all)
          } else {
            (HashSet::new(), false)
          };

          recipe.parameters.iter().enumerate().filter_map(move |(index, parameter)| {
            let used_via_position = uses_all || positional_usage.contains(&(index + 1));

            let is_unused = !identifiers.contains(&parameter.name)
              && !parameter.export
              && !exported
              && !used_via_position;

            is_unused.then(|| {
              Diagnostic::warning(
                format!("Parameter `{}` appears unused", parameter.name),
                parameter.range,
              )
            })
          })
        })
        .collect()
    }
  }
}

impl UnusedRecipeParameterRule {
  fn parse_positional(bytes: &[u8], braced: bool) -> Option<usize> {
    let inner = if braced {
      bytes.strip_prefix(b"{").and_then(|b| b.strip_suffix(b"}"))
    } else {
      Some(bytes)
    }?;

    if inner.is_empty() || !inner.iter().all(u8::is_ascii_digit) {
      return None;
    }

    str::from_utf8(inner).ok()?.parse().ok().filter(|&n| n > 0)
  }

  fn positional_argument_usage(recipe: &Recipe) -> (HashSet<usize>, bool) {
    recipe
      .body
      .iter()
      .flat_map(|line| {
        let bytes = line.value.as_bytes();

        bytes
          .iter()
          .enumerate()
          .filter(move |&(i, _)| {
            bytes[i] == b'$' && (i == 0 || bytes[i - 1] != b'\\')
          })
          .map(move |(i, _)| &bytes[i + 1..])
      })
      .fold((HashSet::new(), false), |(mut indices, uses_all), rest| {
        if matches!(rest.first(), Some(b'@' | b'*'))
          || matches!(rest, [b'{', b'@' | b'*', b'}', ..])
        {
          return (indices, true);
        }

        let unbraced_end =
          rest.iter().take_while(|b| b.is_ascii_digit()).count();

        let index = if unbraced_end > 0 {
          Self::parse_positional(&rest[..unbraced_end], false)
        } else {
          rest
            .iter()
            .position(|&b| b == b'}')
            .and_then(|end| Self::parse_positional(&rest[..=end], true))
        };

        if let Some(index) = index {
          indices.insert(index);
        }

        (indices, uses_all)
      })
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn parse_positional_rejects_incomplete_braced() {
    assert_eq!(
      UnusedRecipeParameterRule::parse_positional(b"{56", true),
      None
    );
  }

  #[test]
  fn parse_positional_rejects_missing_digits() {
    assert_eq!(
      UnusedRecipeParameterRule::parse_positional(b"rest", false),
      None
    );
  }

  #[test]
  fn parse_positional_rejects_zero() {
    assert_eq!(
      UnusedRecipeParameterRule::parse_positional(b"0", false),
      None
    );
  }

  #[test]
  fn parse_positional_with_braces_extracts_number() {
    assert_eq!(
      UnusedRecipeParameterRule::parse_positional(b"{34}", true),
      Some(34)
    );
  }

  #[test]
  fn parse_positional_without_braces_extracts_number() {
    assert_eq!(
      UnusedRecipeParameterRule::parse_positional(b"12", false),
      Some(12)
    );
  }

  #[test]
  fn positional_argument_usage() {
    #[track_caller]
    fn case(body: &str, indices: &[usize], uses_all: bool) {
      let document = Document::from(format!("foo:\n{body}").as_str());

      assert!(!document.tree.root_node().has_error());

      assert_eq!(
        UnusedRecipeParameterRule::positional_argument_usage(
          &document.recipes()[0]
        ),
        (indices.iter().copied().collect::<HashSet<_>>(), uses_all),
      );
    }

    case("", &[], false);
    case("  bar ${3} ${4}", &[3, 4], false);
    case("  bar $1 $2", &[1, 2], false);
    case("  bar $0 $foo ${bar} ${5} \\$6 ${7", &[5], false);
    case("  bar \"${@}\"", &[], true);
    case("  bar ${*}", &[], true);
    case("  bar \"$@\"", &[], true);
    case("  bar $*", &[], true);
    case("  bar \\$@", &[], false);
    case("  bar $1\n  baz $@\n  qux ${3}", &[1, 3], true);
  }
}
