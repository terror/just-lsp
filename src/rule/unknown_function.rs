use super::*;

define_rule! {
  /// Ensures every function call references a builtin function recognized by
  /// `just`.
  UnknownFunctionRule {
    id: "unknown-function",
    message: "unknown function",
    run(context) {
      let mut diagnostics = Vec::new();

      for function_call in context.function_calls() {
        let function_name = &function_call.name.value;

        if context.builtin_function(function_name.as_str()).is_none()
          && !context.user_function_names().contains(function_name)
        {
          let suggestion = function_name.find_suggestion(
            BUILTINS
              .iter()
              .filter_map(|builtin| match builtin {
                Builtin::Function { name, aliases, .. } => {
                  Some(once(*name).chain(aliases.iter().copied()))
                }
                _ => None,
              })
              .flatten()
              .chain(context.user_function_names().iter().map(String::as_str)),
          );

          let message = match &suggestion {
            Some(suggestion) => format!(
              "Unknown function `{function_name}`. Did you mean `{suggestion}`?"
            ),
            None => format!("Unknown function `{function_name}`"),
          };

          let quickfix = suggestion.map(|suggestion| {
            Quickfix::replacement(&function_call.name, suggestion)
          });

          diagnostics.push(
            Diagnostic::error(message, function_call.name.range)
              .quickfix(quickfix),
          );
        }
      }

      diagnostics
    }
  }
}
