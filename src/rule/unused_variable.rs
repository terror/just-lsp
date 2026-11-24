use super::*;

define_rule! {
  /// Finds non-exported global variables that are never referenced in the
  /// document or any of its imports.
  UnusedVariableRule {
    id: "unused-variable",
    message: "unused variable",
    unanimous: true,
    run(context) {
      let mut diagnostics = Vec::new();

      let exported = context.setting_enabled("export");

      for (variable_name, is_used) in &context.references().variable_usage {
        if variable_name.starts_with('_') || *is_used {
          continue;
        }

        let Some(variable) = context
          .local_declarations(context.variables())
          .filter(|variable| variable.name.value == *variable_name)
          .last()
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
