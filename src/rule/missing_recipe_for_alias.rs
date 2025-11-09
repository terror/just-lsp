use super::*;

/// Flags aliases that point to recipes which aren’t defined.
pub struct MissingRecipeForAliasRule;

impl Rule for MissingRecipeForAliasRule {
  fn id(&self) -> &'static str {
    "missing-recipe-for-alias"
  }

  fn display_name(&self) -> &'static str {
    "Missing Recipe for Alias"
  }

  fn run(&self, ctx: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let recipe_names = ctx.recipe_names();

    for alias in ctx.aliases() {
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
