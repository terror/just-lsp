use super::*;

define_rule! {
  /// Flags user-defined functions that reuse the same name multiple times.
  DuplicateFunctionRule {
    id: "duplicate-function",
    message: "duplicate function",
    run(context) {
      let mut groups = HashMap::<String, GroupSet>::new();

      context
        .functions()
        .iter()
        .filter(|function| {
          let current = GroupSet::from_attributes(&function.attributes);
          let previous = groups
            .entry(function.name.value.clone())
            .or_default();
          let duplicate = previous.conflicts_with(&current);

          previous.union_with(current);

          duplicate
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
