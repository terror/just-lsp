use super::*;

/// Ensures each `set` statement uses the correct value type (boolean, string,
/// or array) for the targeted builtin setting.
pub(crate) struct InvalidSettingKindRule;

impl Rule for InvalidSettingKindRule {
  fn display_name(&self) -> &'static str {
    "Invalid Setting Kind"
  }

  fn id(&self) -> &'static str {
    "invalid-setting-kind"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for setting in context.settings() {
      let builtin = BUILTINS.iter().find(
        |f| matches!(f, Builtin::Setting { name, .. } if *name == setting.name),
      );

      let Some(Builtin::Setting { kind, .. }) = builtin else {
        continue;
      };

      if setting.kind == *kind {
        continue;
      }

      diagnostics.push(self.diagnostic(lsp::Diagnostic {
        range: setting.range,
        severity: Some(lsp::DiagnosticSeverity::ERROR),
        message: format!("Setting `{}` expects a {kind} value", setting.name,),
        ..Default::default()
      }));
    }

    diagnostics
  }
}
