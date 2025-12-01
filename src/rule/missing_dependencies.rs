use super::*;

define_rule! {
  /// Reports recipe dependencies that reference recipes which don't exist in the
  /// current document.
  MissingDependencyRule {
    id: "missing-dependencies",
    message: "missing dependency",
    run(ctx) {
      let mut diagnostics = Vec::new();

      let recipe_names = ctx.recipe_names();

      for recipe in ctx.recipes() {
        for dependency in &recipe.dependencies {
          if !recipe_names.contains(&dependency.name) {
            diagnostics.push(Diagnostic::error(
              format!("Recipe `{}` not found", dependency.name),
              dependency.range,
            ));
          }
        }
      }

      diagnostics
    }
  }
}
