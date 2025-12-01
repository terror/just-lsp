use super::*;

define_rule! {
  /// Flags alias declarations that reuse the same name multiple times.
  DuplicateAliasRule {
    id: "duplicate-alias",
    message: "duplicate alias",
    run(ctx) {
      let mut diagnostics = Vec::new();

      let mut seen = HashSet::new();

      for alias in ctx.aliases() {
        if !seen.insert(alias.name.value.clone()) {
          diagnostics.push(Diagnostic::error(
            format!("Duplicate alias `{}`", alias.name.value),
            alias.range,
          ));
        }
      }

      diagnostics
    }
  }
}
