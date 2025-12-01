use super::*;

/// Finds non-exported global variables that are never referenced anywhere in
/// the document.
pub(crate) struct UnusedVariableRule;

impl Rule for UnusedVariableRule {
  fn id(&self) -> &'static str {
    "unused-variables"
  }

  fn message(&self) -> &'static str {
    "unused variable"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    if context.tree().is_none() {
      return diagnostics;
    }

    for (variable_name, is_used) in context.variable_usage() {
      if *is_used {
        continue;
      }

      let Some(variable) = context.document().find_variable(variable_name)
      else {
        continue;
      };

      if variable.export {
        continue;
      }

      diagnostics.push(self.diagnostic(lsp::Diagnostic {
        range: variable.name.range,
        severity: Some(lsp::DiagnosticSeverity::WARNING),
        message: format!("Variable `{variable_name}` appears unused"),
        ..Default::default()
      }));
    }

    diagnostics
  }
}
