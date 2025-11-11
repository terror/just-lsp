use super::*;

/// Finds non-exported global variables that are never referenced anywhere in the
/// document.
pub struct UnusedVariableRule;

impl Rule for UnusedVariableRule {
  fn display_name(&self) -> &'static str {
    "Unused Variables"
  }

  fn id(&self) -> &'static str {
    "unused-variables"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    if context.tree().is_none() {
      return diagnostics;
    }

    for (variable_name, is_used) in context.variable_usage() {
      if !*is_used {
        if let Some(variable) = context.document().find_variable(variable_name)
        {
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
