use super::*;

define_rule! {
  /// Detects recipes that have the same name and overlapping OS constraints,
  /// which would shadow each other at runtime unless overrides are enabled.
  DuplicateRecipeRule {
    id: "duplicate-recipes",
    message: "duplicate recipes",
    run(context) {
      let mut diagnostics = Vec::new();

      let mut groups = HashMap::<String, GroupSet>::new();

      for (recipe, current) in context.recipes_with_groups() {
        let previous = groups
          .entry(recipe.name.value.clone())
          .or_default();

        let overlap = previous.intersection(current);

        previous.union_with(current.clone());

        if !overlap.is_empty()
          && !context.setting_enabled_for("allow-duplicate-recipes", &overlap)
        {
          diagnostics.push(Diagnostic::error(
            format!("Duplicate recipe name `{}`", recipe.name.value),
            recipe.range,
          ));
        }
      }

      diagnostics
    }
  }
}
