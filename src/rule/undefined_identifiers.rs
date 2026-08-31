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
        let message = match suggestion {
          Some(suggestion) => format!(
            "Variable `{}` not found. Did you mean `{suggestion}`?",
            identifier.value,
          ),
          None => format!("Variable `{}` not found", identifier.value),
        };

        let quickfix = suggestion.as_deref().map(|suggestion| {
          Quickfix::replacement(identifier, suggestion)
        });

        diagnostics.push(
          Diagnostic::error(message, identifier.range).quickfix(quickfix),
        );
      }

      diagnostics
    }
  }
}
