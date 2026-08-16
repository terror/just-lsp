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
          deprecated: Some(deprecation),
          ..
        }) = context.builtin_function(function_name.as_str())
        {
          diagnostics.push(Diagnostic::warning(
            format!(
              "`{function_name}` is deprecated, use {deprecation} instead"
            ),
            function_call.name.range,
          ));
        }
      }

      diagnostics
    },
    quickfixes(context) {
      let mut quickfixes = Vec::new();

      for function_call in context.function_calls() {
        let function_name = &function_call.name.value;

        if let Some(Builtin::Function {
          deprecated: Some(Deprecation::Replacement(replacement)),
          ..
        }) = context.builtin_function(function_name.as_str())
        {
          quickfixes.push(Quickfix::replacement(
            &function_call.name,
            *replacement,
          ));
        }
      }

      quickfixes
    }
  }
}
