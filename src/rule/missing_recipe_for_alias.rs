use super::*;

/// Flags aliases that point to recipes which aren’t defined.
pub(crate) struct MissingRecipeForAliasRule;

impl Rule for MissingRecipeForAliasRule {
  fn display_name(&self) -> &'static str {
    "Missing Recipe for Alias"
  }

  fn id(&self) -> &'static str {
    "missing-recipe-for-alias"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let recipe_names = context.recipe_names();

    for alias in context.aliases() {
      if !recipe_names.contains(&alias.value.value) {
        diagnostics.push(self.diagnostic(lsp::Diagnostic {
          range: alias.value.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          message: format!("Recipe `{}` not found", alias.value.value),
          ..Default::default()
        }));
      }
    }

    diagnostics
  }
}
