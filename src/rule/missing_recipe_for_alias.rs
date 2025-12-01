use super::*;

/// Flags aliases that point to recipes which aren’t defined.
pub(crate) struct MissingRecipeForAliasRule;

impl Rule for MissingRecipeForAliasRule {
  fn id(&self) -> &'static str {
    "missing-recipe-for-alias"
  }

  fn message(&self) -> &'static str {
    "alias target not found"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let recipe_names = context.recipe_names();

    for alias in context.aliases() {
      if !recipe_names.contains(&alias.value.value) {
        diagnostics.push(Diagnostic::error(
          format!("Recipe `{}` not found", alias.value.value),
          alias.value.range,
        ));
      }
    }

    diagnostics
  }
}
