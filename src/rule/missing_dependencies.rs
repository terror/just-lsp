use super::*;

/// Reports recipe dependencies that reference recipes which don’t exist in the
/// current document.
pub struct MissingDependencyRule;

impl Rule for MissingDependencyRule {
  fn id(&self) -> &'static str {
    "missing-dependencies"
  }

  fn display_name(&self) -> &'static str {
    "Missing Dependencies"
  }

  fn run(&self, ctx: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let recipe_names = ctx.recipe_names();

    for recipe in ctx.recipes() {
      for dependency in &recipe.dependencies {
        if !recipe_names.contains(&dependency.name) {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: dependency.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!("Recipe `{}` not found", dependency.name),
            ..Default::default()
          }));
        }
      }
    }

    diagnostics
  }
}
