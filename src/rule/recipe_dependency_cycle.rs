use super::*;

define_rule! {
  /// Detects circular dependency chains between recipes to prevent infinite
  /// execution loops.
  RecipeDependencyCycleRule {
    id: "recipe-dependency-cycle",
    message: "circular dependency",
    run(context) {
      let recipes = context.view().resolved_recipes();

      let graph = recipes
        .values()
        .flat_map(|recipe| {
          recipe.dependencies.iter().map(move |dependency| {
            (recipe.name.value.as_str(), dependency.name.value.as_str())
          })
        })
        .collect::<Graph>();

      recipes
        .values()
        .filter(|recipe| recipe.uri == context.document().uri)
        .filter_map(|recipe| {
          let name = &recipe.name.value;

          let cycle = graph.cycle(name)?;

          let message = if cycle.len() == 2 {
            format!("Recipe `{name}` depends on itself")
          } else {
            format!(
              "Recipe `{name}` has circular dependency `{}`",
              cycle.join(" -> ")
            )
          };

          Some(Diagnostic::error(message, recipe.range))
        })
        .collect()
    }
  }
}
