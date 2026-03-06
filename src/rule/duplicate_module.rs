use super::*;

define_rule! {
  /// Flags alias declarations that reuse the same name multiple times.
  DuplicateModuleRule {
    id: "duplicate-module",
    message: "duplicate module",
    run(context) {
      let mut diagnostics = Vec::new();

      let mut seen = HashSet::new();

      for module in context.document().modules() {
        if !seen.insert(module.name.clone()) {
          diagnostics.push(Diagnostic::error(
            format!("Duplicate module `{}`", module.name),
            module.name_range,
          ));
        }
      }

      diagnostics
    }
  }
}
