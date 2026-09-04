use super::*;

define_rule! {
  /// Flags aliases that point to recipes which aren't defined.
  MissingRecipeForAliasRule {
    id: "missing-recipe-for-alias",
    message: "alias target not found",
    run(context) {
      let mut diagnostics = Vec::new();

      let recipe_names = context.recipe_names();

      for alias in context.document().aliases() {
        if !recipe_names.contains(&alias.value.value) {
          let suggestion = alias.value.value.find_suggestion(
            recipe_names.iter().map(String::as_str),
          );

          let message = match &suggestion {
            Some(suggestion) => format!(
              "Recipe `{}` not found. Did you mean `{suggestion}`?",
              alias.value.value,
            ),
            None => format!("Recipe `{}` not found", alias.value.value),
          };

          let quickfix = suggestion.map(|suggestion| {
            Quickfix::replacement(&alias.value, suggestion)
          });

          diagnostics.push(
            Diagnostic::error(message, alias.value.range).quickfix(quickfix),
          );
        }
      }

      diagnostics
    }
  }
}
