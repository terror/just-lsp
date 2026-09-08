use super::*;

define_rule! {
  /// Flags user-defined functions that reuse the same name multiple times.
  DuplicateFunctionRule {
    id: "duplicate-function",
    message: "duplicate function",
    run(context) {
      let mut conflicts = ConflictTracker::default();

      context
        .functions()
        .iter()
        .filter(|function| {
          conflicts.record(&function.name, &function.attributes)
        })
        .map(|function| {
          Diagnostic::error(
            format!("Duplicate function `{}`", function.name.value),
            function.range,
          )
        })
        .collect()
    }
  }
}
