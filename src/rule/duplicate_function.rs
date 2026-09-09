use super::*;

define_rule! {
  /// Flags user-defined functions that reuse the same name multiple times.
  DuplicateFunctionRule {
    id: "duplicate-function",
    message: "duplicate function",
    run(context) {
      context
        .duplicate_declarations(
          context.functions(),
          |function| &function.name,
          |function| &function.attributes,
        )
        .into_iter()
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
