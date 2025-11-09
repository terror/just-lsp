use super::*;

pub struct SettingsRule;

impl Rule for SettingsRule {
  fn id(&self) -> &'static str {
    "settings"
  }

  fn display_name(&self) -> &'static str {
    "Settings"
  }

  fn run(&self, ctx: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let settings = ctx.settings();

    for setting in settings {
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
      } else {
        diagnostics.push(self.diagnostic(lsp::Diagnostic {
          range: setting.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          message: format!("Unknown setting `{}`", setting.name),
          ..Default::default()
        }));
      }
    }

    let mut seen = HashSet::new();

    for setting in settings {
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
