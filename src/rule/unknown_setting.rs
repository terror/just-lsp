use super::*;

/// Emits diagnostics for `set` directives targeting settings that don’t exist
/// in the builtin catalog.
pub(crate) struct UnknownSettingRule;

impl Rule for UnknownSettingRule {
  fn id(&self) -> &'static str {
    "unknown-setting"
  }

  fn message(&self) -> &'static str {
    "unknown setting"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<Diagnostic> {
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
