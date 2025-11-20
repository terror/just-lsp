use super::*;

/// Warn when `[parallel]` is applied to a recipe that lacks enough
/// dependencies for the attribute to have any effect.
pub(crate) struct ParallelDependenciesRule;

impl Rule for ParallelDependenciesRule {
  fn display_name(&self) -> &'static str {
    "Parallel Dependencies"
  }

  fn id(&self) -> &'static str {
    "parallel-dependencies"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    context
      .recipes()
      .iter()
      .filter_map(|recipe| {
        let attribute = recipe.find_attribute("parallel")?;

        let message = match recipe.dependencies.len() {
          0 => format!(
            "Recipe `{}` has no dependencies, so `[parallel]` has no effect",
            recipe.name
          ),
          1 => format!(
            "Recipe `{}` has only one dependency, so `[parallel]` has no effect",
            recipe.name
          ),
          _ => return None,
        };

        Some(self.diagnostic(lsp::Diagnostic {
          message,
          range: attribute.range,
          severity: Some(lsp::DiagnosticSeverity::WARNING),
          ..Default::default()
        }))
      })
      .collect()
  }
}
