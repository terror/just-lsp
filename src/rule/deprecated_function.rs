use super::*;

define_rule! {
  /// Warns when a deprecated function is used and suggests the replacement.
  DeprecatedFunctionRule {
    id: "deprecated-function",
    message: "deprecated function",
    run(context) {
      let mut diagnostics = Vec::new();

      for function_call in context.function_calls() {
        let function_name = &function_call.name.value;

        if let Some(Builtin::Function {
          deprecated: Some(replacement),
          ..
        }) = context.builtin_function(function_name.as_str())
        {
          diagnostics.push(Diagnostic::warning(
            format!(
              "`{function_name}` is deprecated, use `{replacement}` instead"
            ),
            function_call.name.range,
          ));
        }
      }

      diagnostics
    }
  }
}
