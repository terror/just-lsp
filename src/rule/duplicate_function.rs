use super::*;

define_rule! {
  /// Flags user-defined functions that reuse the same name multiple times.
  DuplicateFunctionRule {
    id: "duplicate-function",
    message: "duplicate function",
    run(context) {
      let mut groups = HashMap::<String, GroupSet>::new();

      context
        .functions_with_groups()
        .filter(|(function, current)| {
          let previous = groups
            .entry(function.name.value.clone())
            .or_default();

          let duplicate = previous.conflicts_with(current);

          previous.union_with((*current).clone());

          duplicate
        })
        .map(|(function, _)| {
          Diagnostic::error(
            format!("Duplicate function `{}`", function.name.value),
            function.range,
          )
        })
        .collect()
    }
  }
}
