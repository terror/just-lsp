use super::*;

/// Ensures every function call references a builtin function recognized by
/// `just`.
pub(crate) struct UnknownFunctionRule;

impl Rule for UnknownFunctionRule {
  fn id(&self) -> &'static str {
    "unknown-function"
  }

  fn message(&self) -> &'static str {
    "Unknown Function"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for function_call in context.function_calls() {
      let function_name = &function_call.name.value;

      if context.builtin_function(function_name.as_str()).is_none() {
        diagnostics.push(self.diagnostic(lsp::Diagnostic {
          range: function_call.name.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          message: format!("Unknown function `{function_name}`"),
          ..Default::default()
        }));
      }
    }

    diagnostics
  }
}
