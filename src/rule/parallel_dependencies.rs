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

          let diagnostic = match recipe.dependencies.len() {
            0 => Diagnostic::warning(
              format!(
                "Recipe `{}` has no dependencies, so `[parallel]` has no effect",
                recipe.name.value
              ),
              attribute.range,
            ),
            1 => Diagnostic::warning(
              format!(
                "Recipe `{}` has only one dependency, so `[parallel]` has no effect",
                recipe.name.value
              ),
              attribute.range,
            ),
            _ => return None,
          };

          Some(diagnostic.quickfix(Quickfix::removal(
            attribute.range,
            "Remove `[parallel]`",
          )))
        })
        .collect()
    }
  }
}
