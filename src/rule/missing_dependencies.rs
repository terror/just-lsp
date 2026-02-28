use super::*;

define_rule! {
  /// Reports recipe dependencies that reference recipes which don't exist in the
  /// current document.
  MissingDependencyRule {
    id: "missing-dependencies",
    message: "missing dependency",
    run(context) {
      let mut diagnostics = Vec::new();

      let recipe_names = context.recipe_names();

      for recipe in context.recipes() {
        for dependency in &recipe.dependencies {
          if dependency.name.contains("::")
            || recipe_names.contains(&dependency.name)
          {
            continue;
          }

          diagnostics.push(Diagnostic::error(
            format!("Recipe `{}` not found", dependency.name),
            dependency.range,
          ));
        }
      }

      diagnostics
    }
  }
}
