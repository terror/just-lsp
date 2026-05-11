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

        if let Some(Builtin::Function { kind, .. }) =
          context.builtin_function(function_name.as_str())
        {
          let range = kind.argument_range();

          let (min, max) = (*range.start(), *range.end());

          let argument_count = function_call.arguments.len();

          if argument_count < min {
            diagnostics.push(Diagnostic::error(
              format!(
                "Function `{function_name}` requires at least {min} {}, but {argument_count} provided",
                Count("argument", min)
              ),
              function_call.range,
            ));
          } else if argument_count > max {
            let upper = if min == max {
              format!("{max} {}", Count("argument", max))
            } else {
              format!("at most {max} {}", Count("argument", max))
            };

            diagnostics.push(Diagnostic::error(
              format!(
                "Function `{function_name}` accepts {upper}, but {argument_count} provided"
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
