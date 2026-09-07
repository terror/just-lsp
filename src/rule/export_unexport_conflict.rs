use super::*;

define_rule! {
  ExportUnexportConflictRule {
    id: "export-unexport-conflict",
    message: "export/unexport conflict",
    run(context) {
      let mut unexports = ConflictTracker::default();

      for unexport in context.unexports() {
        unexports.record(&unexport.name, &unexport.attributes);
      }

      let mut diagnostics = Vec::new();

      for variable in context.variables() {
        if unexports.conflicts_with(&variable.name, &variable.attributes) {
          diagnostics.push(Diagnostic::error(
            format!(
              "Variable {} is both exported and unexported",
              variable.name.value
            ),
            variable.name.range,
          ));
        }
      }

      diagnostics
    }
  }
}
