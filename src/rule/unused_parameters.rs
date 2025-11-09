use super::*;

pub struct UnusedParameterRule;

impl Rule for UnusedParameterRule {
  fn id(&self) -> &'static str {
    "unused-parameters"
  }

  fn display_name(&self) -> &'static str {
    "Unused Parameters"
  }

  fn run(&self, ctx: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let exported = ctx.settings().iter().any(|setting| {
      setting.name == "export" && setting.kind == SettingKind::Boolean(true)
    });

    for (recipe_name, identifiers) in ctx.recipe_identifier_usage() {
      if let Some(recipe) = ctx.recipe(recipe_name) {
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
