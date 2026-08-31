use super::*;

define_rule! {
  /// Reports recipe dependencies that reference recipes which don't exist in the
  /// current document.
  MissingDependencyRule {
    id: "missing-dependencies",
    message: "missing dependency",
    provides_quickfixes: true,
    run(context) {
      let mut diagnostics = Vec::new();

      let recipe_names = context.recipe_names();

      for recipe in context.document().recipes() {
        for dependency in &recipe.dependencies {
          if !recipe_names.contains(&dependency.name.value) {
            let suggestion = dependency.name.value.find_suggestion(
              recipe_names.iter().map(String::as_str),
            );

            let message = match &suggestion {
              Some(suggestion) => format!(
                "Recipe `{}` not found. Did you mean `{suggestion}`?",
                dependency.name.value,
              ),
              None => format!("Recipe `{}` not found", dependency.name.value),
            };

            let quickfix = suggestion.map(|suggestion| {
              Quickfix::replacement(&dependency.name, suggestion)
            });

            diagnostics.push(
              Diagnostic::error(message, dependency.range).quickfix(quickfix),
            );
          }
        }
      }

      diagnostics
    }
  }
}
