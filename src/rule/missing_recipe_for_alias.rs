use super::*;

define_rule! {
  /// Flags aliases that point to recipes which aren't defined.
  MissingRecipeForAliasRule {
    id: "missing-recipe-for-alias",
    message: "alias target not found",
    run(ctx) {
      let mut diagnostics = Vec::new();

      let recipe_names = ctx.recipe_names();

      for alias in ctx.aliases() {
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
}
