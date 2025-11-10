use super::*;

/// Emits diagnostics for `set` directives targeting settings that don’t exist in
/// the builtin catalog.
pub struct UnknownSettingRule;

impl Rule for UnknownSettingRule {
  fn id(&self) -> &'static str {
    "unknown-setting"
  }

  fn display_name(&self) -> &'static str {
    "Unknown Setting"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for setting in context.settings() {
      let builtin = builtins::BUILTINS.iter().find(
        |f| matches!(f, Builtin::Setting { name, .. } if *name == setting.name),
      );

      if builtin.is_none() {
        diagnostics.push(self.diagnostic(lsp::Diagnostic {
          range: setting.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          message: format!("Unknown setting `{}`", setting.name),
          ..Default::default()
        }));
      }
    }

    diagnostics
  }
}
