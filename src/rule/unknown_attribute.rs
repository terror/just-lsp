use super::*;

/// Warns when an attribute name isn’t part of the known builtin attribute set.
pub struct UnknownAttributeRule;

impl Rule for UnknownAttributeRule {
  fn display_name(&self) -> &'static str {
    "Unknown Attribute"
  }

  fn id(&self) -> &'static str {
    "unknown-attribute"
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
        diagnostics.push(self.diagnostic(lsp::Diagnostic {
          range: attribute.name.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          message: format!("Unknown attribute `{attribute_name}`"),
          ..Default::default()
        }));
      }
    }

    diagnostics
  }
}
