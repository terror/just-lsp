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
    let mut diagnostics = Vec::new();

    for recipe in context.recipes() {
      let Some(attribute) = recipe.find_attribute("parallel") else {
        continue;
      };

      match recipe.dependencies.len() {
        0 | 1 => {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: attribute.range,
            severity: Some(lsp::DiagnosticSeverity::WARNING),
            message: Self::message(recipe.dependencies.len(), recipe),
            ..Default::default()
          }));
        }
        _ => {}
      }
    }

    diagnostics
  }
}

impl ParallelDependenciesRule {
  fn message(count: usize, recipe: &Recipe) -> String {
    match count {
      0 => format!(
        "Recipe `{}` has no dependencies, so `[parallel]` has no effect",
        recipe.name
      ),
      1 => format!(
        "Recipe `{}` has only one dependency, so `[parallel]` has no effect",
        recipe.name
      ),
      _ => unreachable!(
        "parallel dependency warning only applies to 0 or 1 dependencies"
      ),
    }
  }
}
