use super::*;

define_rule! {
  /// Flags user-defined functions that reuse the same name multiple times.
  DuplicateFunctionRule {
    id: "duplicate-function",
    message: "duplicate function",
    run(context) {
      let mut diagnostics = Vec::new();
      let mut seen = HashSet::new();

      for function in context.functions() {
        if !seen.insert(function.name.value.clone()) {
          diagnostics.push(Diagnostic::error(
            format!("Duplicate function `{}`", function.name.value),
            function.range,
          ));
        }
      }

      diagnostics
    }
  }
}
