use super::*;

define_rule! {
  /// Highlights recipe parameters that never get read anywhere in the recipe body
  /// (unless `set export` is on).
  UnusedParameterRule {
    id: "unused-parameters",
    message: "unused parameter",
    run(context) {
      let exported = context.setting_enabled("export");

      let positional_arguments_enabled = context.setting_enabled("positional-arguments");

      context
        .recipe_identifier_usage()
        .iter()
        .filter_map(|(recipe_name, identifiers)| {
          context.recipe(recipe_name).map(|recipe| (recipe, identifiers))
        })
        .flat_map(|(recipe, identifiers): (&Recipe, _)| {
          let recipe_enables_positional_arguments =
            positional_arguments_enabled || recipe.has_attribute("positional-arguments");

          let (positional_usage, uses_all) = if recipe_enables_positional_arguments {
            (
              UnusedParameterRule::positional_argument_indices(recipe),
              UnusedParameterRule::uses_all_positional_arguments(recipe),
            )
          } else {
            (HashSet::new(), false)
          };

          recipe.parameters.iter().enumerate().filter_map(move |(index, parameter)| {
            let used_via_position = uses_all || positional_usage.contains(&(index + 1));

            let is_unused = !identifiers.contains(&parameter.name)
              && parameter.kind != ParameterKind::Export
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

impl UnusedParameterRule {
  fn is_unescaped_dollar(bytes: &[u8], i: usize) -> bool {
    bytes[i] == b'$' && (i == 0 || bytes[i - 1] != b'\\')
  }

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

  fn positional_argument_indices(recipe: &Recipe) -> HashSet<usize> {
    let bytes = recipe.content.as_bytes();

    bytes
      .iter()
      .enumerate()
      .filter(|&(i, _)| Self::is_unescaped_dollar(bytes, i))
      .filter_map(|(i, _)| {
        let rest = &bytes[i + 1..];

        let unbraced_end =
          rest.iter().take_while(|b| b.is_ascii_digit()).count();

        if unbraced_end > 0 {
          return Self::parse_positional(&rest[..unbraced_end], false);
        }

        let brace_end = rest.iter().position(|&b| b == b'}')?;

        Self::parse_positional(&rest[..=brace_end], true)
      })
      .collect()
  }

  fn uses_all_positional_arguments(recipe: &Recipe) -> bool {
    let bytes = recipe.content.as_bytes();

    bytes.iter().enumerate().any(|(i, _)| {
      if !Self::is_unescaped_dollar(bytes, i) {
        return false;
      }

      let rest = &bytes[i + 1..];

      matches!(rest.first(), Some(b'@' | b'*'))
        || matches!(rest, [b'{', b'@' | b'*', b'}', ..])
    })
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq, std::collections::HashSet};

  fn recipe(content: &str) -> Recipe {
    Recipe {
      attributes: vec![],
      content: content.into(),
      dependencies: vec![],
      name: TextNode {
        value: "graph".into(),
        range: lsp::Range::default(),
      },
      parameters: vec![],
      range: lsp::Range::default(),
      shebang: None,
    }
  }

  #[test]
  fn positional_argument_indices_detects_unbraced_arguments() {
    assert_eq!(
      UnusedParameterRule::positional_argument_indices(&recipe(
        "graph log:\n  ./bin/graph $1 $2 text"
      )),
      HashSet::from([1, 2])
    );
  }

  #[test]
  fn positional_argument_indices_detects_braced_arguments() {
    assert_eq!(
      UnusedParameterRule::positional_argument_indices(&recipe(
        "graph log:\n  ./bin/graph ${3} ${4}"
      )),
      HashSet::from([3, 4])
    );
  }

  #[test]
  fn positional_argument_indices_ignores_invalid_variants() {
    assert_eq!(
      UnusedParameterRule::positional_argument_indices(&recipe(
        "graph log:\n  echo $0 $foo ${bar} ${5} \\$6 ${7"
      )),
      HashSet::from([5])
    );
  }

  #[test]
  fn parse_positional_without_braces_extracts_number() {
    assert_eq!(
      UnusedParameterRule::parse_positional(b"12", false),
      Some(12)
    );
  }

  #[test]
  fn parse_positional_with_braces_extracts_number() {
    assert_eq!(
      UnusedParameterRule::parse_positional(b"{34}", true),
      Some(34)
    );
  }

  #[test]
  fn parse_positional_rejects_missing_digits() {
    assert_eq!(UnusedParameterRule::parse_positional(b"rest", false), None);
  }

  #[test]
  fn parse_positional_rejects_incomplete_braced() {
    assert_eq!(UnusedParameterRule::parse_positional(b"{56", true), None);
  }

  #[test]
  fn parse_positional_rejects_zero() {
    assert_eq!(UnusedParameterRule::parse_positional(b"0", false), None);
  }

  #[test]
  fn uses_all_positional_arguments_detects_dollar_at() {
    assert!(UnusedParameterRule::uses_all_positional_arguments(&recipe(
      "run *args:\n  echo \"$@\""
    )));
  }

  #[test]
  fn uses_all_positional_arguments_detects_dollar_star() {
    assert!(UnusedParameterRule::uses_all_positional_arguments(&recipe(
      "run *args:\n  echo $*"
    )));
  }

  #[test]
  fn uses_all_positional_arguments_detects_braced_at() {
    assert!(UnusedParameterRule::uses_all_positional_arguments(&recipe(
      "run *args:\n  echo \"${@}\""
    )));
  }

  #[test]
  fn uses_all_positional_arguments_detects_braced_star() {
    assert!(UnusedParameterRule::uses_all_positional_arguments(&recipe(
      "run *args:\n  echo ${*}"
    )));
  }

  #[test]
  fn uses_all_positional_arguments_ignores_escaped() {
    assert!(!UnusedParameterRule::uses_all_positional_arguments(
      &recipe("run *args:\n  echo \\$@")
    ));
  }

  #[test]
  fn uses_all_positional_arguments_returns_false_when_absent() {
    assert!(!UnusedParameterRule::uses_all_positional_arguments(
      &recipe("run *args:\n  echo $1 $2")
    ));
  }
}
