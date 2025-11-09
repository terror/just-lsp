use super::*;

pub struct UnusedVariableRule;

impl Rule for UnusedVariableRule {
  fn id(&self) -> &'static str {
    "unused-variables"
  }

  fn display_name(&self) -> &'static str {
    "Unused Variables"
  }

  fn run(&self, ctx: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    if ctx.tree().is_none() {
      return diagnostics;
    }

    for (variable_name, is_used) in ctx.variable_usage() {
      if !*is_used {
        if let Some(variable) = ctx.document().find_variable(variable_name) {
          if !variable.export {
            diagnostics.push(self.diagnostic(lsp::Diagnostic {
              range: variable.name.range,
              severity: Some(lsp::DiagnosticSeverity::WARNING),
              message: format!("Variable `{variable_name}` appears unused"),
              ..Default::default()
            }));
          }
        }
      }
    }

    diagnostics
  }
}
