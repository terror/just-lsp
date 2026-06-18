use super::*;

define_rule! {
  ExportUnexportConflictRule {
    id: "export-unexport-conflict",
    message: "export/unexport conflict",
    run(context) {
      let unexports = context
        .unexports()
        .iter()
        .map(|unexport| unexport.name.value.clone())
        .collect::<HashSet<_>>();

      let mut diagnostics = Vec::new();

      for variable in context.variables() {
        if unexports.contains(&variable.name.value) {
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
