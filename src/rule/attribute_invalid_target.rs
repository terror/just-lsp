use super::*;

/// Ensures attributes only appear on syntax nodes that actually accept
/// attributes.
pub(crate) struct AttributeInvalidTargetRule;

impl Rule for AttributeInvalidTargetRule {
  fn id(&self) -> &'static str {
    "attribute-invalid-target"
  }

  fn message(&self) -> &'static str {
    "Attribute Invalid Target"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for attribute in context.attributes() {
      let attribute_name = &attribute.name.value;

      if context.builtin_attributes(attribute_name).is_empty() {
        continue;
      }

      if attribute.target.is_none() {
        diagnostics.push(self.diagnostic(lsp::Diagnostic {
          range: attribute.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          message: format!(
            "Attribute `{attribute_name}` applied to invalid target",
          ),
          ..Default::default()
        }));
      }
    }

    diagnostics
  }
}
