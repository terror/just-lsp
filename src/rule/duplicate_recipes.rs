use super::*;

define_rule! {
  /// Detects recipes that have the same name and overlapping OS constraints,
  /// which would shadow each other at runtime unless overrides are enabled.
  DuplicateRecipeRule {
    id: "duplicate-recipes",
    message: "duplicate recipes",
    run(context) {
      let allow_duplicates = context.setting_enabled("allow-duplicate-recipes");

      if allow_duplicates {
        return Vec::new();
      }

      let mut diagnostics = Vec::new();

      for recipe in context.duplicate_declarations(
        context.recipes(),
        |recipe| &recipe.name,
        |recipe| &recipe.attributes,
      ) {
        diagnostics.push(Diagnostic::error(
          format!("Duplicate recipe name `{}`", recipe.name.value),
          recipe.range,
        ));
      }

      diagnostics
    }
  }
}
