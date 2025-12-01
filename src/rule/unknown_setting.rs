use super::*;

define_rule! {
  /// Emits diagnostics for `set` directives targeting settings that don't exist
  /// in the builtin catalog.
  UnknownSettingRule {
    id: "unknown-setting",
    message: "unknown setting",
    run(ctx) {
      let mut diagnostics = Vec::new();

      for setting in ctx.settings() {
        if ctx.builtin_setting(&setting.name).is_none() {
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
