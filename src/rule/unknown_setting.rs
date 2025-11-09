use super::*;

pub struct UnknownSettingRule;

impl Rule for UnknownSettingRule {
  fn id(&self) -> &'static str {
    "unknown-setting"
  }

  fn display_name(&self) -> &'static str {
    "Unknown Setting"
  }

  fn run(&self, ctx: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for setting in ctx.settings() {
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
