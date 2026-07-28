use super::*;

define_rule! {
  /// Reports expressions that reference variables or parameters which are not
  /// defined and aren't builtins.
  UndefinedIdentifierRule {
    id: "undefined-identifiers",
    message: "undefined identifier",
    phase: RulePhase::Project,
    run(context) {
      let mut diagnostics = Vec::new();

      for (name, range) in &context.scope().unresolved_identifiers {
        diagnostics.push(Diagnostic::error(
          format!("Variable `{name}` not found"),
          *range,
        ));
      }

      diagnostics
    }
  }
}
