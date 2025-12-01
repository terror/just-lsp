use super::*;

define_rule! {
  /// Validates recipe parameter lists for duplicate names, ordering mistakes, and
  /// illegal variadic/default combinations.
  RecipeParameterRule {
    id: "recipe-parameters",
    message: "invalid recipe parameters",
    run(context) {
      let mut diagnostics = Vec::new();

      for recipe in context.recipes() {
        let mut seen = HashSet::new();

        let (mut passed_default, mut passed_variadic) = (false, false);

        for (index, param) in recipe.parameters.iter().enumerate() {
          if !seen.insert(param.name.clone()) {
            diagnostics.push(Diagnostic::error(
              format!("Duplicate parameter `{}`", param.name),
              param.range,
            ));
          }

          let has_default = param.default_value.is_some();

          if matches!(param.kind, ParameterKind::Variadic(_)) {
            if index < recipe.parameters.len() - 1 {
              diagnostics.push(Diagnostic::error(
                format!(
                  "Variadic parameter `{}` must be the last parameter",
                  param.name
                ),
                param.range,
              ));
            }

            passed_variadic = true;
          }

          if passed_default
            && !has_default
            && !matches!(param.kind, ParameterKind::Variadic(_))
          {
            diagnostics.push(Diagnostic::error(
              format!(
                "Required parameter `{}` follows a parameter with a default value",
                param.name
              ),
              param.range,
            ));
          }

          if passed_variadic && index < recipe.parameters.len() - 1 {
            diagnostics.push(Diagnostic::error(
              format!("Parameter `{}` follows a variadic parameter", param.name),
              param.range,
            ));
          }

          if has_default {
            passed_default = true;
          }
        }
      }

      diagnostics
    }
  }
}
