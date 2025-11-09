use super::*;

pub struct DependencyArgumentRule;

impl Rule for DependencyArgumentRule {
  fn id(&self) -> &'static str {
    "dependency-arguments"
  }

  fn display_name(&self) -> &'static str {
    "Dependency Arguments"
  }

  fn run(&self, ctx: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let recipe_parameters = ctx.recipe_parameters();

    for recipe in ctx.recipes() {
      for dependency in &recipe.dependencies {
        if let Some(params) = recipe_parameters.get(&dependency.name) {
          let required_params = params
            .iter()
            .filter(|p| {
              p.default_value.is_none()
                && !matches!(p.kind, ParameterKind::Variadic(_))
            })
            .count();

          let has_variadic = params
            .iter()
            .any(|p| matches!(p.kind, ParameterKind::Variadic(_)));

          let total_params = params.len();
          let arg_count = dependency.arguments.len();

          if arg_count < required_params {
            diagnostics.push(self.diagnostic(lsp::Diagnostic {
              range: dependency.range,
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              message: format!(
                "Dependency `{}` requires {required_params} {}, but {arg_count} provided",
                dependency.name,
                Count("argument", required_params)
              ),
              ..Default::default()
            }));
          } else if !has_variadic && arg_count > total_params {
            diagnostics.push(self.diagnostic(lsp::Diagnostic {
              range: dependency.range,
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              message: format!(
                "Dependency `{}` accepts {total_params} {}, but {arg_count} provided",
                dependency.name,
                Count("argument", total_params)
              ),
              ..Default::default()
            }));
          }
        }
      }
    }

    diagnostics
  }
}
