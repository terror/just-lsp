use super::*;

define_rule! {
  /// Warns when a deprecated setting is used and suggests the replacement.
  DeprecatedSettingRule {
    id: "deprecated-setting",
    message: "deprecated setting",
    run(context) {
      let mut diagnostics = Vec::new();

      for setting in context.local_declarations(context.settings()) {
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
              attribute,
              setting: replacement,
              value,
            } => match setting.kind {
              SettingKind::Boolean(false) => diagnostic.quickfix(
                Quickfix::removal(
                  setting.range,
                  format!("Remove `set {}`", setting.name.value),
                )
              ),
              SettingKind::Boolean(true) => diagnostic.quickfix(
                Self::attribute_quickfix(
                  context,
                  setting,
                  attribute,
                  replacement,
                  lsp::TextEdit {
                    range: lsp::Range::new(
                      setting.name.range.start,
                      setting.value.range.end,
                    ),
                    new_text: format!("{replacement} := {value}"),
                  },
                )
              ),
              _ => diagnostic,
            },
            Deprecation::Replacement(replacement) => {
              diagnostic.quickfix(Quickfix::replacement(&setting.name, replacement))
            }
            Deprecation::SettingAttribute {
              attribute,
              setting: replacement,
            } => diagnostic.quickfix(
              Self::attribute_quickfix(
                context,
                setting,
                attribute,
                replacement,
                lsp::TextEdit {
                  range: setting.name.range,
                  new_text: replacement.into(),
                },
              )
            ),
          };

          diagnostics.push(diagnostic);
        }
      }

      diagnostics
    }
  }
}

impl DeprecatedSettingRule {
  fn attribute_quickfix(
    context: &RuleContext<'_>,
    setting: &Setting,
    attribute: &str,
    replacement: &str,
    edit: lsp::TextEdit,
  ) -> Option<Quickfix> {
    let groups = GroupSet::from_attribute(attribute)?;

    let has_other_platform = setting.attributes.iter().any(|candidate| {
      candidate.name.value != attribute && candidate.condition().is_some()
    });

    let replacement_exists = context.settings().iter().any(|candidate| {
      candidate.name.value == replacement
        && GroupSet::from_attributes(&candidate.attributes)
          .conflicts_with(&groups)
    });

    if has_other_platform || replacement_exists {
      return None;
    }

    let attribute_edit =
      (!setting.has_attribute(attribute)).then(|| lsp::TextEdit {
        range: lsp::Range::new(setting.range.start, setting.range.start),
        new_text: format!("[{attribute}]\n"),
      });

    Some(Quickfix::new(
      format!(
        "Replace `{}` with `[{attribute}] set {replacement}`",
        setting.name.value
      ),
      [edit].into_iter().chain(attribute_edit),
    ))
  }
}
