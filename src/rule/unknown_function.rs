use {super::*, crate::suggestion::suggest};

define_rule! {
  /// Ensures every function call references a builtin function recognized by
  /// `just`.
  UnknownFunctionRule {
    id: "unknown-function",
    message: "unknown function",
    provides_quickfixes: true,
    run(context) {
      let mut diagnostics = Vec::new();

      for function_call in context.function_calls() {
        let function_name = &function_call.name.value;

        if context.builtin_function(function_name.as_str()).is_none()
          && !context.user_function_names().contains(function_name)
        {
          let suggestion = suggest(
            function_name,
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

          let mut diagnostic = Diagnostic::error(
            match &suggestion {
              Some(suggestion) => format!(
                "Unknown function `{function_name}`\nDid you mean `{suggestion}`?"
              ),
              None => format!("Unknown function `{function_name}`"),
            },
            function_call.name.range,
          );

          if let Some(suggestion) = suggestion {
            diagnostic = diagnostic.quickfix(Quickfix::replacement(
              &function_call.name,
              suggestion,
            ));
          }

          diagnostics.push(diagnostic);
        }
      }

      diagnostics
    }
  }
}
