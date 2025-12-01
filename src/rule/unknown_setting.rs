use super::*;

define_rule! {
  /// Emits diagnostics for `set` directives targeting settings that don't exist
  /// in the builtin catalog.
  UnknownSettingRule {
    id: "unknown-setting",
    message: "unknown setting",
    run(context) {
      let mut diagnostics = Vec::new();

      for setting in context.settings() {
        if context.builtin_setting(&setting.name).is_none() {
          diagnostics.push(Diagnostic::error(
            format!("Unknown setting `{}`", setting.name),
            setting.range,
          ));
        }
      }

      diagnostics
    }
  }
}
