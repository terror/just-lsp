use super::*;

define_rule! {
  /// Ensures each `set` statement uses the correct value type (boolean, string,
  /// or array) for the targeted builtin setting.
  InvalidSettingKindRule {
    id: "invalid-setting-kind",
    message: "invalid setting kind",
    run(context) {
      let mut diagnostics = Vec::new();

      for setting in context.settings() {
        let Some(Builtin::Setting { kind, .. }) =
          context.builtin_setting(&setting.name.value)
        else {
          continue;
        };

        let list_dotenv_setting = matches!(
          setting.name.value.as_str(),
          "dotenv-filename" | "dotenv-path"
        ) && context.setting_enabled("lists")
          && setting.kind == SettingKind::Array;

        if setting.kind == *kind || list_dotenv_setting {
          continue;
        }

        let article = if *kind == SettingKind::Array {
          "an"
        } else {
          "a"
        };

        diagnostics.push(Diagnostic::error(
          format!(
            "Setting `{}` expects {article} {kind} value",
            setting.name.value
          ),
          setting.range,
        ));
      }

      diagnostics
    }
  }
}
