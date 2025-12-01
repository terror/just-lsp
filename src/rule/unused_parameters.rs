use super::*;

/// Highlights recipe parameters that never get read anywhere in the recipe body
/// (unless `set export` is on).
pub(crate) struct UnusedParameterRule;

impl Rule for UnusedParameterRule {
  fn id(&self) -> &'static str {
    "unused-parameters"
  }

  fn message(&self) -> &'static str {
    "unused parameters"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let exported = context.setting_enabled("export");

    let positional_arguments_enabled =
      context.setting_enabled("positional-arguments");

    for (recipe_name, identifiers) in context.recipe_identifier_usage() {
      if let Some(recipe) = context.recipe(recipe_name) {
        let recipe_enables_positional_arguments = positional_arguments_enabled
          || recipe.has_attribute("positional-arguments");

        let positional_argument_usage = if recipe_enables_positional_arguments {
          Self::positional_argument_indices(recipe)
        } else {
          HashSet::new()
        };

        for (index, parameter) in recipe.parameters.iter().enumerate() {
          let used_via_position =
            positional_argument_usage.contains(&(index + 1));

          if !identifiers.contains(&parameter.name)
            && parameter.kind != ParameterKind::Export
            && !exported
            && !used_via_position
          {
            diagnostics.push(self.diagnostic(lsp::Diagnostic {
              range: parameter.range,
              severity: Some(lsp::DiagnosticSeverity::WARNING),
              message: format!("Parameter `{}` appears unused", parameter.name),
              ..Default::default()
            }));
          }
        }
      }
    }

    diagnostics
  }
}

impl UnusedParameterRule {
  fn parse_digits(
    bytes: &[u8],
    mut index: usize,
    len: usize,
    expect_braces: bool,
  ) -> Option<(usize, usize)> {
    if expect_braces {
      if index >= len || bytes[index] != b'{' {
        return None;
      }

      index += 1;
    }

    let start = index;

    while index < len && bytes[index].is_ascii_digit() {
      index += 1;
    }

    if start == index {
      return None;
    }

    if expect_braces {
      if index >= len || bytes[index] != b'}' {
        return None;
      }

      Some((
        (index - start) + 2,
        str::from_utf8(&bytes[start..index]).ok()?.parse().ok()?,
      ))
    } else {
      Some((
        index - start,
        str::from_utf8(&bytes[start..index]).ok()?.parse().ok()?,
      ))
    }
  }

  fn positional_argument_indices(recipe: &Recipe) -> HashSet<usize> {
    let mut indices = HashSet::new();

    let bytes = recipe.content.as_bytes();
    let len = bytes.len();

    let mut i = 0;

    while i < len {
      if bytes[i] == b'$' && (i == 0 || bytes[i - 1] != b'\\') {
        if let Some((advance, value)) =
          Self::parse_digits(bytes, i + 1, len, false)
        {
          if value > 0 {
            indices.insert(value);
          }

          i += advance + 1;

          continue;
        }

        if let Some((advance, value)) =
          Self::parse_digits(bytes, i + 1, len, true)
        {
          if value > 0 {
            indices.insert(value);
          }

          i += advance + 1;

          continue;
        }
      }

      i += 1;
    }

    indices
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
      name: "graph".into(),
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
  fn parse_digits_without_braces_extracts_number() {
    let bytes = b"12 rest";

    assert_eq!(
      UnusedParameterRule::parse_digits(bytes, 0, bytes.len(), false),
      Some((2, 12))
    );
  }

  #[test]
  fn parse_digits_with_braces_extracts_number() {
    let bytes = b"{34} rest";

    assert_eq!(
      UnusedParameterRule::parse_digits(bytes, 0, bytes.len(), true),
      Some((4, 34))
    );
  }

  #[test]
  fn parse_digits_rejects_missing_digits() {
    let bytes = b"rest";

    assert_eq!(
      UnusedParameterRule::parse_digits(bytes, 0, bytes.len(), false),
      None
    );
  }

  #[test]
  fn parse_digits_rejects_incomplete_braced() {
    let bytes = b"{56";

    assert_eq!(
      UnusedParameterRule::parse_digits(bytes, 0, bytes.len(), true),
      None
    );
  }
}
