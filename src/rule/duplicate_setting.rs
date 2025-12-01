use super::*;

/// Emits diagnostics when the same `set` option is declared more than once.
pub(crate) struct DuplicateSettingRule;

impl Rule for DuplicateSettingRule {
  fn id(&self) -> &'static str {
    "duplicate-setting"
  }

  fn message(&self) -> &'static str {
    "Duplicate Setting"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let mut seen = HashSet::new();

    for setting in context.settings() {
      if !seen.insert(setting.name.clone()) {
        diagnostics.push(self.diagnostic(lsp::Diagnostic {
          range: setting.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          message: format!("Duplicate setting `{}`", setting.name),
          ..Default::default()
        }));
      }
    }

    diagnostics
  }
}
