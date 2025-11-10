use super::*;

/// Verifies builtin function calls use a valid argument count and respect
/// variadic constraints.
pub struct FunctionArgumentsRule;

impl Rule for FunctionArgumentsRule {
  fn display_name(&self) -> &'static str {
    "Function Arguments"
  }

  fn id(&self) -> &'static str {
    "function-arguments"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for function_call in context.function_calls() {
      let function_name = &function_call.name.value;

      let builtin = builtins::BUILTINS.iter().find(|f| {
        matches!(f, Builtin::Function { name, .. } if *name == function_name)
      });

      if let Some(Builtin::Function {
        required_args,
        accepts_variadic,
        ..
      }) = builtin
      {
        let arg_count = function_call.arguments.len();

        if arg_count < *required_args {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: function_call.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!(
              "Function `{function_name}` requires at least {required_args} {}, but {arg_count} provided",
              Count("argument", *required_args)
            ),
            ..Default::default()
          }));
        } else if !accepts_variadic && arg_count > *required_args {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: function_call.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!(
              "Function `{function_name}` accepts {required_args} {}, but {arg_count} provided",
              Count("argument", *required_args)
            ),
            ..Default::default()
          }));
        }
      }
    }

    diagnostics
  }
}
