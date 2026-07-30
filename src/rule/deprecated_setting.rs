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
          let diagnostic = Diagnostic::warning(
            format!(
              "`{}` is deprecated, use {deprecation} instead",
              setting.name.value
            ),
            setting.name.range,
          );

          let diagnostic = match *deprecation {
            Deprecation::Replacement(replacement) => {
              diagnostic.quickfix(Quickfix::replacement(&setting.name, replacement))
            }
            Deprecation::SettingAttribute {
              attribute,
              setting: replacement,
            } => {
              if context.settings().iter().any(|replacement_setting| {
                replacement_setting.name.value == replacement
                  && replacement_setting.has_attribute(attribute)
              }) {
                diagnostic
              } else {
                diagnostic.quickfix(Quickfix::setting_attribute(
                  setting,
                  context.document(),
                  attribute,
                  replacement,
                ))
              }
            }
          };

          diagnostics.push(diagnostic);
        }
      }

      diagnostics
    }
  }
}
