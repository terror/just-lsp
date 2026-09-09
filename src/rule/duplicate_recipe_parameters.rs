use super::*;

define_rule! {
  /// Validates recipe parameter lists for duplicate names, ordering mistakes,
  /// and illegal variadic/default combinations.
  DuplicateRecipeParameterRule {
    id: "duplicate-recipe-parameters",
    message: "invalid recipe parameters",
    run(context) {
      let mut diagnostics = Vec::new();

      for recipe in context.local_declarations(context.recipes()) {
        let mut seen = HashSet::new();

        let mut passed_default = false;

        for param in &recipe.parameters {
          if !seen.insert(param.name.clone()) {
            diagnostics.push(Diagnostic::error(
              format!("Duplicate parameter `{}`", param.name),
              param.range,
            ));
          }

          let has_default = param.default_value.is_some();

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

          if has_default {
            passed_default = true;
          }
        }
      }

      diagnostics
    }
  }
}
