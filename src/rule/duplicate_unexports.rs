use super::*;

define_rule! {
  DuplicateUnexportRule {
    id: "duplicate-unexport",
    message: "duplicate unexport",
    run(context) {
      let mut diagnostics = Vec::new();

      let mut groups = HashMap::<String, GroupSet>::new();

      for (unexport, current) in context.unexports_with_groups() {
        let previous = groups
          .entry(unexport.name.value.clone())
          .or_default();

        let duplicate = previous.conflicts_with(current);

        previous.union_with(current.clone());

        if duplicate {
          diagnostics.push(Diagnostic::error(
            format!(
              "Variable `{}` is unexported multiple times",
              unexport.name.value
            ),
            unexport.name.range,
          ));
        }
      }

      diagnostics
    }
  }
}
