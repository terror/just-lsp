use super::*;

/// Reports expressions that reference variables or parameters which are not
/// defined and aren’t builtins.
pub(crate) struct UndefinedIdentifierRule;

impl Rule for UndefinedIdentifierRule {
  fn id(&self) -> &'static str {
    "undefined-identifiers"
  }

  fn message(&self) -> &'static str {
    "Undefined Identifiers"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for unresolved in context.unresolved_identifiers() {
      diagnostics.push(self.diagnostic(lsp::Diagnostic {
        range: unresolved.range,
        severity: Some(lsp::DiagnosticSeverity::ERROR),
        message: format!("Variable `{}` not found", unresolved.name),
        ..Default::default()
      }));
    }

    diagnostics
  }
}
