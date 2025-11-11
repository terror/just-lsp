use super::*;

/// Ensures every function call references a builtin function recognized by
/// `just`.
pub struct UnknownFunctionRule;

impl Rule for UnknownFunctionRule {
  fn display_name(&self) -> &'static str {
    "Unknown Function"
  }

  fn id(&self) -> &'static str {
    "unknown-function"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for function_call in context.function_calls() {
      let function_name = &function_call.name.value;

      let is_builtin = BUILTINS.iter().any(|f| {
        matches!(f, Builtin::Function { name, .. } if *name == function_name)
      });

      if !is_builtin {
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
