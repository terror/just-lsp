use super::*;

/// Detects conflicts between working-directory and no-cd directives.
pub(crate) struct WorkingDirectoryConflictRule;

impl Rule for WorkingDirectoryConflictRule {
  fn id(&self) -> &'static str {
    "working-directory-conflict"
  }

  fn message(&self) -> &'static str {
    "working directory conflict"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for recipe in context.recipes() {
      let working_directory_attribute =
        recipe.find_attribute("working-directory");

      let no_cd_attribute = recipe.find_attribute("no-cd");

      if let (Some(attribute), Some(_)) =
        (working_directory_attribute, no_cd_attribute)
      {
        diagnostics.push(self.diagnostic(lsp::Diagnostic {
          range: attribute.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          message: format!(
            "Recipe `{}` can't combine `[working-directory]` with `[no-cd]`",
            recipe.name
          ),
          ..Default::default()
        }));
      }
    }

    diagnostics
  }
}
