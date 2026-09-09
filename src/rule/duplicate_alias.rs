use super::*;

define_rule! {
  /// Flags alias declarations that reuse the same name multiple times.
  DuplicateAliasRule {
    id: "duplicate-alias",
    message: "duplicate alias",
    run(context) {
      let mut diagnostics = Vec::new();

      for alias in context.duplicate_declarations(
        context.aliases(),
        |alias| &alias.name,
        |alias| &alias.attributes,
      ) {
        diagnostics.push(Diagnostic::error(
          format!("Duplicate alias `{}`", alias.name.value),
          alias.range,
        ));
      }

      diagnostics
    }
  }
}
