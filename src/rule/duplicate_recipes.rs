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

      let mut groups = HashMap::<String, GroupSet>::new();

      for recipe in context.recipes() {
        let current = GroupSet::from_attributes(&recipe.attributes);

        let previous = groups
          .entry(recipe.name.value.clone())
          .or_default();

        let duplicate = previous.conflicts_with(&current);

        previous.union_with(current);

        if duplicate {
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
