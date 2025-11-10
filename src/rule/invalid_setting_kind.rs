use super::*;

/// Ensures each `set` statement uses the correct value type (boolean, string, or
/// array) for the targeted builtin setting.
pub struct InvalidSettingKindRule;

impl Rule for InvalidSettingKindRule {
  fn id(&self) -> &'static str {
    "invalid-setting-kind"
  }

  fn display_name(&self) -> &'static str {
    "Invalid Setting Kind"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for setting in context.settings() {
      let builtin = builtins::BUILTINS.iter().find(
        |f| matches!(f, Builtin::Setting { name, .. } if *name == setting.name),
      );

      if let Some(Builtin::Setting { kind, .. }) = builtin {
        if setting.kind != *kind {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: setting.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!(
              "Setting `{}` expects a {kind} value",
              setting.name,
            ),
            ..Default::default()
          }));
        }
      }
    }

    diagnostics
  }
}
