use super::*;

/// Ensures attributes only appear on syntax nodes that actually accept attributes.
pub struct AttributeInvalidTargetRule;

impl Rule for AttributeInvalidTargetRule {
  fn display_name(&self) -> &'static str {
    "Attribute Invalid Target"
  }

  fn id(&self) -> &'static str {
    "attribute-invalid-target"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for attribute in context.attributes() {
      let attribute_name = &attribute.name.value;

      let is_known = builtins::BUILTINS.iter().any(|f| {
        matches!(
          f,
          Builtin::Attribute { name, .. } if *name == attribute_name.as_str()
        )
      });

      if !is_known {
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
