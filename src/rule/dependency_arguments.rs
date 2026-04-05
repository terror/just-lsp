use super::*;

define_rule! {
  /// Checks that dependency invocations supply the correct number of arguments
  /// for the referenced recipe's signature.
  DependencyArgumentRule {
    id: "dependency-arguments",
    message: "invalid dependency arguments",
    run(context) {
      let mut diagnostics = Vec::new();

      let recipe_parameters = context.recipe_parameters();

      for recipe in context.recipes() {
        for dependency in &recipe.dependencies {
          if let Some(parameters) = recipe_parameters.get(&dependency.name) {
            let required_parameters = parameters
              .iter()
              .filter(|parameter| {
                parameter.default_value.is_none()
                  && !matches!(
                    parameter.kind,
                    ParameterKind::Variadic(VariadicType::ZeroOrMore)
                  )
              })
              .count();

            let has_variadic = parameters
              .iter()
              .any(|parameter| matches!(parameter.kind, ParameterKind::Variadic(_)));

            let (argument_count, parameter_count) = (dependency.arguments.len(), parameters.len());

            if argument_count < required_parameters {
              diagnostics.push(Diagnostic::error(
                format!(
                  "Dependency `{}` requires {required_parameters} {}, but {argument_count} provided",
                  dependency.name,
                  Count("argument", required_parameters)
                ),
                dependency.range,
              ));
            } else if !has_variadic && argument_count > parameter_count {
              diagnostics.push(Diagnostic::error(
                format!(
                  "Dependency `{}` accepts {parameter_count} {}, but {argument_count} provided",
                  dependency.name,
                  Count("argument", parameter_count)
                ),
                dependency.range,
              ));
            }
          }
        }
      }

      diagnostics
    }
  }
}
