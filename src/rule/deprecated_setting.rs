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
          deprecated: Some(deprecation),
          ..
        }) = context.builtin_setting(&setting.name.value)
        {
          diagnostics.push(Diagnostic::warning(
            format!(
              "`{}` is deprecated, use {deprecation} instead",
              setting.name.value
            ),
            setting.name.range,
          ));
        }
      }

      diagnostics
    },
    quickfixes(context) {
      let mut quickfixes = Vec::new();
      for setting in context.settings() {
        if let Some(Builtin::Setting {
          deprecated: Some(deprecation),
          ..
        }) = context.builtin_setting(&setting.name.value)
        {
          let deprecation = *deprecation;

          let quickfix = match deprecation {
            Deprecation::Replacement(replacement) => {
              Quickfix::replacement(&setting.name, replacement)
            }
            Deprecation::SettingAttribute {
              attribute,
              setting: replacement,
            } => {
              if context.settings().iter().any(|replacement_setting| {
                replacement_setting.name.value == replacement
                  && replacement_setting
                    .has_attribute(context.attributes(), attribute)
              }) {
                continue;
              }

              Quickfix::setting_attribute(
                setting,
                context.document(),
                attribute,
                replacement,
              )
            }
          };

          quickfixes.push(quickfix);
        }
      }

      quickfixes
    }
  }
}
