use super::*;

/// Reports recipe dependencies that reference recipes which don’t exist in the
/// current document.
pub(crate) struct MissingDependencyRule;

impl Rule for MissingDependencyRule {
  fn id(&self) -> &'static str {
    "missing-dependencies"
  }

  fn message(&self) -> &'static str {
    "missing dependency"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let recipe_names = context.recipe_names();

    for recipe in context.recipes() {
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
