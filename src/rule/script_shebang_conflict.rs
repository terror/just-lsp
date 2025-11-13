use super::*;

/// Reports recipes that combine a shebang line with the `[script]` attribute.
pub(crate) struct ScriptShebangConflictRule;

impl Rule for ScriptShebangConflictRule {
  fn display_name(&self) -> &'static str {
    "Script Shebang Conflict"
  }

  fn id(&self) -> &'static str {
    "script-shebang-conflict"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for recipe in context.recipes() {
      let Some(script_attribute) = recipe.find_attribute("script") else {
        continue;
      };

      if recipe.shebang.is_none() {
        continue;
      }

      diagnostics.push(self.diagnostic(lsp::Diagnostic {
        range: script_attribute.range,
        severity: Some(lsp::DiagnosticSeverity::ERROR),
        message: format!(
          "Recipe `{}` has both shebang line and `[script]` attribute",
          recipe.name
        ),
        ..Default::default()
      }));
    }

    diagnostics
  }
}
