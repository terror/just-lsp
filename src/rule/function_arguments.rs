use super::*;

define_rule! {
  /// Verifies builtin function calls use a valid argument count and respect
  /// variadic constraints.
  FunctionArgumentsRule {
    id: "function-arguments",
    message: "invalid function arguments",
    run(ctx) {
      let mut diagnostics = Vec::new();

      for function_call in ctx.function_calls() {
        let function_name = &function_call.name.value;

        if let Some(Builtin::Function {
          required_args,
          accepts_variadic,
          ..
        }) = ctx.builtin_function(function_name.as_str())
        {
          let arg_count = function_call.arguments.len();

          if arg_count < *required_args {
            diagnostics.push(Diagnostic::error(
              format!(
                "Function `{function_name}` requires at least {required_args} {}, but {arg_count} provided",
                Count("argument", *required_args)
              ),
              function_call.range,
            ));
          } else if !accepts_variadic && arg_count > *required_args {
            diagnostics.push(Diagnostic::error(
              format!(
                "Function `{function_name}` accepts {required_args} {}, but {arg_count} provided",
                Count("argument", *required_args)
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
