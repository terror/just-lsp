use super::*;

define_rule! {
  /// Highlights recipe parameters that never get read anywhere in the recipe
  /// body (unless `set export` is on).
  UnusedRecipeParameterRule {
    id: "unused-recipe-parameter",
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

enum Slice {
  All,
  Malformed,
  Positions(Vec<usize>),
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

  /// Resolves a `${@:N}`, `${*:N}`, `${@:N:M}` or `${*:N:M}` slice, given the
  /// text following the `@` or `*`. Slices that can't be resolved statically
  /// are treated as using every parameter.
  fn parse_slice(rest: &[u8], parameters: usize) -> Slice {
    let Some(end) = rest.iter().position(|&b| b == b'}') else {
      return Slice::Malformed;
    };

    let Some(inner) = str::from_utf8(&rest[..end])
      .ok()
      .and_then(|text| text.strip_prefix(':'))
    else {
      return Slice::Malformed;
    };

    let mut parts = inner.splitn(2, ':');

    let offset = parts.next().unwrap_or_default();

    let length = parts.next();

    let literal = |text: &str| {
      let text = text.trim();

      (!text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()))
        .then(|| text.parse::<usize>().ok())
        .flatten()
    };

    let Some(offset) = literal(offset) else {
      return Slice::All;
    };

    let upper = match length {
      None => {
        if offset == 0 {
          return Slice::All;
        }

        parameters
      }
      Some(length) => match literal(length) {
        Some(length) => offset.saturating_add(length).saturating_sub(1),
        None if offset == 0 => return Slice::All,
        None => parameters,
      },
    };

    Slice::Positions((offset.max(1)..=upper.min(parameters)).collect())
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

        if let [b'{', b'@' | b'*', tail @ ..] = rest {
          match Self::parse_slice(tail, recipe.parameters.len()) {
            Slice::Positions(positions) => indices.extend(positions),
            Slice::All => return (indices, true),
            Slice::Malformed => {}
          }

          return (indices, uses_all);
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
      let document = Document::from(format!("foo a b c:\n{body}").as_str());

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
    case("  bar \"${@:1}\"", &[1, 2, 3], false);
    case("  bar \"${@:2}\"", &[2, 3], false);
    case("  bar \"${*:3}\"", &[3], false);
    case("  bar \"${@:4}\"", &[], false);
    case("  bar \"${@:0}\"", &[], true);
    case("  bar \"${@:2:1}\"", &[2], false);
    case("  bar \"${@:2:2}\"", &[2, 3], false);
    case("  bar \"${*:2:2}\"", &[2, 3], false);
    case("  bar \"${@:2:0}\"", &[], false);
    case("  bar \"${@:0:2}\"", &[1], false);
    case("  bar \"${@:2:$n}\"", &[2, 3], false);
    case("  bar \\${@:2}", &[], false);
    case("  bar \"${@:$n}\"", &[], true);
    case("  bar \"${@: -1}\"", &[], true);
    case("  bar \"${@:x}\"", &[], true);
    case("  bar \"${@:2", &[], false);
  }
}
