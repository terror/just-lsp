use super::*;

define_rule! {
  /// Warns when a deprecated setting is used and suggests the replacement.
  DeprecatedSettingRule {
    id: "deprecated-setting",
    message: "deprecated setting",
    run(context) {
      let mut diagnostics = Vec::new();

      for setting in context.document().settings() {
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
              let has_other_platform = setting.attributes.iter().any(|candidate| {
                candidate.name.value != attribute
                  && candidate.condition().is_some()
              });

              let replacement_exists = context.settings().iter().any(|candidate| {
                candidate.name.value == replacement
                  && candidate.has_attribute(attribute)
              });

              if has_other_platform || replacement_exists {
                diagnostic
              } else {
                let mut edits = vec![lsp::TextEdit {
                  range: setting.name.range,
                  new_text: replacement.into(),
                }];

                if !setting.has_attribute(attribute) {
                  edits.push(lsp::TextEdit {
                    range: lsp::Range::new(setting.range.start, setting.range.start),
                    new_text: format!("[{attribute}]\n"),
                  });
                }

                diagnostic.quickfix(
                  Quickfix::new(
                    format!(
                      "Replace `{}` with `[{attribute}] set {replacement}`",
                      setting.name.value
                    ),
                    edits,
                  )
                )
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
