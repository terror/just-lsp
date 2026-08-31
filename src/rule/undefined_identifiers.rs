use super::*;

define_rule! {
  /// Reports expressions that reference variables or parameters which are not
  /// defined and aren't builtins.
  UndefinedIdentifierRule {
    id: "undefined-identifiers",
    message: "undefined identifier",
    provides_quickfixes: true,
    run(context) {
      let mut diagnostics = Vec::new();

      for (identifier, suggestion) in &context.scope().unresolved_identifiers {
        let mut diagnostic = Diagnostic::error(
          match suggestion {
            Some(suggestion) => format!(
              "Variable `{}` not found. Did you mean `{suggestion}`?",
              identifier.value,
            ),
            None => format!("Variable `{}` not found", identifier.value),
          },
          identifier.range,
        );

        if let Some(suggestion) = suggestion {
          diagnostic = diagnostic
            .quickfix(Quickfix::replacement(identifier, suggestion));
        }

        diagnostics.push(diagnostic);
      }

      diagnostics
    }
  }
}
