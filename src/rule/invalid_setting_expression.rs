use super::*;

define_rule! {
  InvalidSettingExpressionRule {
    id: "invalid-setting-expression",
    message: "invalid setting expression",
    run(context) {
      let mut diagnostics = Vec::new();
      let mut expressions = ConstExpressions::new(context.view());

      for setting in context.local_declarations(context.settings()) {
        if matches!(
          setting.name.value.as_str(),
          "dotenv-command" | "dotenv-filename" | "dotenv-path"
            | "script-interpreter" | "shell" | "tempdir" | "windows-shell"
            | "working-directory"
        ) && expressions.is_non_const(&setting.value) {
          diagnostics.push(Diagnostic::error(
            format!(
              "Setting `{}` must be a const expression",
              setting.name.value,
            ),
            setting.value.range,
          ));
        }
      }

      diagnostics
    }
  }
}
