use super::*;

define_rule! {
  /// Finds non-exported global variables that are never referenced anywhere in
  /// the document.
  UnusedVariableRule {
    id: "unused-variables",
    message: "unused variable",
    run(context) {
      let mut diagnostics = Vec::new();

      if context.tree().is_none() {
        return diagnostics;
      }

      let exported = context.setting_enabled("export");

      for (variable_name, is_used) in &context.scope().variable_usage {
        if *is_used {
          continue;
        }

        let Some(variable) = context.document().find_variable(variable_name)
        else {
          continue;
        };

        if variable.export || exported {
          continue;
        }

        diagnostics.push(Diagnostic::warning(
          format!("Variable `{variable_name}` appears unused"),
          variable.name.range,
        ));
      }

      diagnostics
    }
  }
}
