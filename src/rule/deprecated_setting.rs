use super::*;

define_rule! {
  /// Warns when a deprecated setting is used and suggests the replacement.
  DeprecatedSettingRule {
    id: "deprecated-setting",
    message: "deprecated setting",
    run(context) {
      let mut diagnostics = Vec::new();

      for setting in context.settings() {
        if let Some(Builtin::Setting {
          deprecated: Some(replacement),
          ..
        }) = context.builtin_setting(&setting.name)
        {
          diagnostics.push(Diagnostic::warning(
            format!(
              "`{}` is deprecated, use `{replacement}` instead",
              setting.name
            ),
            setting.range,
          ));
        }
      }

      diagnostics
    }
  }
}
