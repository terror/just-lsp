use super::*;

/// Validates that each attribute is attached to a supported target kind
/// (recipe, module, alias, etc.) according to the builtin metadata.
pub(crate) struct AttributeTargetSupportRule;

impl Rule for AttributeTargetSupportRule {
  fn id(&self) -> &'static str {
    "attribute-target-support"
  }

  fn message(&self) -> &'static str {
    "unsupported attribute target"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for attribute in context.attributes() {
      let attribute_name = &attribute.name.value;

      let matching = context.builtin_attributes(attribute_name);

      if matching.is_empty() {
        continue;
      }

      let Some(target_type) = attribute.target else {
        continue;
      };

      let is_valid_target = matching.iter().copied().any(|attr| {
        if let Builtin::Attribute { targets, .. } = attr {
          targets.contains(&target_type)
        } else {
          false
        }
      });

      if !is_valid_target {
        diagnostics.push(self.diagnostic(lsp::Diagnostic {
          range: attribute.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          message: format!(
            "Attribute `{attribute_name}` cannot be applied to {target_type} target",
          ),
          ..Default::default()
        }));
      }
    }

    diagnostics
  }
}
