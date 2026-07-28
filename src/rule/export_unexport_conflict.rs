use super::*;

define_rule! {
  ExportUnexportConflictRule {
    id: "export-unexport-conflict",
    message: "export/unexport conflict",
    run(context) {
      let mut unexports = HashMap::<String, GroupSet>::new();

      for (unexport, current) in context.unexports_with_groups() {
        unexports
          .entry(unexport.name.value.clone())
          .or_default()
          .union_with(current.clone());
      }

      let mut diagnostics = Vec::new();

      for (variable, current) in context.variables_with_groups() {
        if unexports
          .get(&variable.name.value)
          .is_some_and(|previous| previous.conflicts_with(current))
        {
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
