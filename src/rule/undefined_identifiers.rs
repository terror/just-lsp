use super::*;

define_rule! {
  /// Reports expressions that reference variables or parameters which are not
  /// defined and aren't builtins.
  UndefinedIdentifierRule {
    id: "undefined-identifiers",
    message: "undefined identifier",
    run(ctx) {
      let mut diagnostics = Vec::new();

      for unresolved in ctx.unresolved_identifiers() {
        diagnostics.push(Diagnostic::error(
          format!("Variable `{}` not found", unresolved.name),
          unresolved.range,
        ));
      }

      diagnostics
    }
  }
}
