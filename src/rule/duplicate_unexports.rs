use super::*;

define_rule! {
  DuplicateUnexportRule {
    id: "duplicate-unexport",
    message: "duplicate unexport",
    run(context) {
      let mut diagnostics = Vec::new();

      let mut conflicts = ConflictTracker::default();

      for unexport in context.unexports() {
        if conflicts.record(&unexport.name, &unexport.attributes) {
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
