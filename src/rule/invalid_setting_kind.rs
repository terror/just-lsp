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
          context.builtin_setting(&setting.name)
        else {
          continue;
        };

        if setting.kind == *kind {
          continue;
        }

        diagnostics.push(Diagnostic::error(
          format!("Setting `{}` expects a {kind} value", setting.name,),
          setting.range,
        ));
      }

      diagnostics
    }
  }
}
