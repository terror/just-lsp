use super::*;

define_rule! {
  /// Warn when `[parallel]` is applied to a recipe that lacks enough
  /// dependencies for the attribute to have any effect.
  ParallelDependenciesRule {
    id: "parallel-dependencies",
    message: "unnecessary parallel attribute",
    run(context) {
      context
        .recipes()
        .iter()
        .filter_map(|recipe| {
          let attribute = recipe.find_attribute("parallel")?;

          let message = match recipe.dependencies.len() {
            0 => format!(
              "Recipe `{}` has no dependencies, so `[parallel]` has no effect",
              recipe.name.value
            ),
            1 => format!(
              "Recipe `{}` has only one dependency, so `[parallel]` has no effect",
              recipe.name.value
            ),
            _ => return None,
          };

          Some(Diagnostic::warning(message, attribute.range))
        })
        .collect()
    }
  }
}
