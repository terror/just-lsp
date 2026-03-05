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

      let mut recipe_groups: HashMap<String, Vec<(lsp::Range, HashSet<Group>)>> =
        HashMap::new();

      for recipe in context.recipes() {
        recipe_groups
          .entry(recipe.name.value.clone())
          .or_default()
          .push((recipe.range, recipe.groups()));
      }

      for (recipe_name, group) in &recipe_groups {
        if group.len() <= 1 {
          continue;
        }

        for (i, (range, a)) in group.iter().enumerate() {
          for (_, (_, b)) in group.iter().enumerate().take(i) {
            let has_conflict =
              a.iter().any(|a| b.iter().any(|b| a.conflicts_with(*b)));

            if has_conflict {
              diagnostics.push(Diagnostic::error(
                format!("Duplicate recipe name `{recipe_name}`"),
                *range,
              ));

              break;
            }
          }
        }
      }

      diagnostics
    }
  }
}
