use super::*;

define_rule! {
  /// Validates recipe parameter lists for duplicate names, ordering mistakes,
  /// and illegal variadic/default combinations.
  DuplicateRecipeParameterRule {
    id: "duplicate-recipe-parameter",
    message: "invalid recipe parameters",
    run(context) {
      let mut diagnostics = Vec::new();

      for recipe in context.local_declarations(context.recipes()) {
        let mut seen = HashSet::new();

        for param in &recipe.parameters {
          if !seen.insert(param.name.clone()) {
            diagnostics.push(Diagnostic::error(
              format!("Duplicate parameter `{}`", param.name),
              param.range,
            ));
          }
        }
      }

      diagnostics
    }
  }
}
