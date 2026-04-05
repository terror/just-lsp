use super::*;

define_rule! {
  /// Verifies builtin function calls use a valid argument count and respect
  /// variadic constraints.
  FunctionArgumentsRule {
    id: "function-arguments",
    message: "invalid function arguments",
    run(context) {
      let mut diagnostics = Vec::new();

      for function_call in context.function_calls() {
        let function_name = &function_call.name.value;

        if let Some(Builtin::Function {
          required_arguments,
          accepts_variadic,
          ..
        }) = context.builtin_function(function_name.as_str())
        {
          let argument_count = function_call.arguments.len();

          if argument_count < *required_arguments {
            diagnostics.push(Diagnostic::error(
              format!(
                "Function `{function_name}` requires at least {required_arguments} {}, but {argument_count} provided",
                Count("argument", *required_arguments)
              ),
              function_call.range,
            ));
          } else if !accepts_variadic && argument_count > *required_arguments {
            diagnostics.push(Diagnostic::error(
              format!(
                "Function `{function_name}` accepts {required_arguments} {}, but {argument_count} provided",
                Count("argument", *required_arguments)
              ),
              function_call.range,
            ));
          }
        } else if let Some(function) = context
          .functions()
          .iter()
          .find(|function| function.name.value == *function_name)
        {
          let (argument_count, parameter_count) = (function_call.arguments.len(), function.parameters.len());

          if argument_count != parameter_count {
            diagnostics.push(Diagnostic::error(
              format!(
                "Function `{function_name}` accepts {parameter_count} {}, but {argument_count} provided",
                Count("argument", parameter_count)
              ),
              function_call.range,
            ));
          }
        }
      }

      diagnostics
    }
  }
}
