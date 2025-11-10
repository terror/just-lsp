use super::*;

/// Highlights recipe parameters that never get read anywhere in the recipe body
/// (unless `set export` is on).
pub struct UnusedParameterRule;

impl Rule for UnusedParameterRule {
  fn id(&self) -> &'static str {
    "unused-parameters"
  }

  fn display_name(&self) -> &'static str {
    "Unused Parameters"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let exported = context.settings().iter().any(|setting| {
      setting.name == "export"
        && matches!(setting.kind, SettingKind::Boolean(true))
    });

    for (recipe_name, identifiers) in context.recipe_identifier_usage() {
      if let Some(recipe) = context.recipe(recipe_name) {
        for parameter in &recipe.parameters {
          if !identifiers.contains(&parameter.name)
            && parameter.kind != ParameterKind::Export
            && !exported
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
