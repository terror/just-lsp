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
            Deprecation::BooleanSetting {
              setting: replacement,
              value,
            } => {
              let groups = GroupSet::from_attributes(&setting.attributes);

              let replacement_exists = context.settings().iter().any(|candidate| {
                candidate.name.value == replacement
                  && GroupSet::from_attributes(&candidate.attributes)
                    .conflicts_with(&groups)
              });

              match setting.kind {
                SettingKind::Boolean(false) => diagnostic.quickfix(
                  Quickfix::removal(
                    setting.range,
                    format!("Remove `set {}`", setting.name.value),
                  )
                ),
                SettingKind::Boolean(true) if !replacement_exists => diagnostic.quickfix(
                  Quickfix::edit(
                    format!("Replace `{}` with `{replacement}`", setting.name.value),
                    lsp::Range::new(setting.name.range.start, setting.value.range.end),
                    format!("{replacement} := {value}"),
                  )
                ),
                _ => diagnostic,
              }
            }
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
