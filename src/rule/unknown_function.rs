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

        if context.builtin_function(function_name.as_str()).is_none() {
          diagnostics.push(Diagnostic::error(
            format!("Unknown function `{function_name}`"),
            function_call.name.range,
          ));
        }
      }

      diagnostics
    }
  }
}
