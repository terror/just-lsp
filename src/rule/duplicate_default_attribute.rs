use super::*;

/// Flags multiple `[default]` attributes within the same module (document).
pub(crate) struct DuplicateDefaultAttributeRule;

impl Rule for DuplicateDefaultAttributeRule {
  fn display_name(&self) -> &'static str {
    "Duplicate Default Attribute"
  }

  fn id(&self) -> &'static str {
    "duplicate-default-attribute"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut has_default_attribute = false;

    for recipe in context.recipes() {
      for attribute in &recipe.attributes {
        if attribute.name.value == "default" {
          if has_default_attribute {
            diagnostics.push(self.diagnostic(lsp::Diagnostic {
              range: attribute.range,
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              message: format!(
                "Recipe `{}` has duplicate `[default]` attribute, which may only appear once per module",
                recipe.name
              ),
              ..Default::default()
            }));
          } else {
            has_default_attribute = true;
          }
        }
      }
    }

    diagnostics
  }
}
