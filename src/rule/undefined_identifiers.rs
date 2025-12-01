use super::*;

/// Reports expressions that reference variables or parameters which are not
/// defined and aren’t builtins.
pub(crate) struct UndefinedIdentifierRule;

impl Rule for UndefinedIdentifierRule {
  fn id(&self) -> &'static str {
    "undefined-identifiers"
  }

  fn message(&self) -> &'static str {
    "undefined identifier"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for unresolved in context.unresolved_identifiers() {
      diagnostics.push(Diagnostic::error(
        format!("Variable `{}` not found", unresolved.name),
        unresolved.range,
      ));
    }

    diagnostics
  }
}
