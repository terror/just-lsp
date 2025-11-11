use super::*;

/// Validates recipe parameter lists for duplicate names, ordering mistakes, and
/// illegal variadic/default combinations.
pub struct RecipeParameterRule;

impl Rule for RecipeParameterRule {
  fn display_name(&self) -> &'static str {
    "Recipe Parameters"
  }

  fn id(&self) -> &'static str {
    "recipe-parameters"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for recipe in context.recipes() {
      let mut seen = HashSet::new();

      let (mut passed_default, mut passed_variadic) = (false, false);

      for (index, param) in recipe.parameters.iter().enumerate() {
        if !seen.insert(param.name.clone()) {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: param.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!("Duplicate parameter `{}`", param.name),
            ..Default::default()
          }));
        }

        let has_default = param.default_value.is_some();

        if matches!(param.kind, ParameterKind::Variadic(_)) {
          if index < recipe.parameters.len() - 1 {
            diagnostics.push(self.diagnostic(lsp::Diagnostic {
              range: param.range,
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              message: format!(
                "Variadic parameter `{}` must be the last parameter",
                param.name
              ),
              ..Default::default()
            }));
          }

          passed_variadic = true;
        }

        if passed_default
          && !has_default
          && !matches!(param.kind, ParameterKind::Variadic(_))
        {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: param.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!(
              "Required parameter `{}` follows a parameter with a default value",
              param.name
            ),
            ..Default::default()
          }));
        }

        if passed_variadic && index < recipe.parameters.len() - 1 {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: param.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!(
              "Parameter `{}` follows a variadic parameter",
              param.name
            ),
            ..Default::default()
          }));
        }

        if has_default {
          passed_default = true;
        }
      }
    }

    diagnostics
  }
}
