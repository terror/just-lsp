use super::*;

define_rule! {
  DuplicateUnexportRule {
    id: "duplicate-unexport",
    message: "duplicate unexport",
    phase: RulePhase::Project,
    run(context) {
      let mut diagnostics = Vec::new();

      let mut groups = HashMap::<String, GroupSet>::new();

      for unexport in context.unexports() {
        let current = GroupSet::from_attributes(&unexport.attributes);

        let previous = groups
          .entry(unexport.name.value.clone())
          .or_default();

        let duplicate = previous.conflicts_with(&current);

        previous.union_with(current);

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
