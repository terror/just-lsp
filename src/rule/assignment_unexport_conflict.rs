use super::*;

define_rule! {
  AssignmentUnexportConflictRule {
    id: "assignment-unexport-conflict",
    message: "export/unexport conflict",
    run(context) {
      let mut unexports = ConflictTracker::default();

      for unexport in context.unexports() {
        unexports.record(&unexport.name, &unexport.attributes);
      }

      let mut diagnostics = Vec::new();

      for variable in context.local_declarations(context.variables()) {
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

      let mut variables = ConflictTracker::default();

      for variable in context.variables() {
        if variable.uri != context.document().uri {
          variables.record(&variable.name, &variable.attributes);
        }
      }

      for unexport in context.local_declarations(context.unexports()) {
        if variables.conflicts_with(&unexport.name, &unexport.attributes) {
          diagnostics.push(Diagnostic::error(
            format!(
              "Variable {} is both exported and unexported",
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
