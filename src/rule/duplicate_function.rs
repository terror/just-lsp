use super::*;

define_rule! {
  /// Flags user-defined functions that reuse the same name multiple times.
  DuplicateFunctionRule {
    id: "duplicate-function",
    message: "duplicate function",
    run(context) {
      let mut seen = HashSet::new();

      context
        .functions()
        .iter()
        .filter(|function| !seen.insert(function.name.value.clone()))
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
