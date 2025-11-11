use super::*;

/// Validates that each attribute is attached to a supported target kind (recipe,
/// module, alias, etc.) according to the builtin metadata.
pub(crate) struct AttributeTargetSupportRule;

impl Rule for AttributeTargetSupportRule {
  fn display_name(&self) -> &'static str {
    "Attribute Target Support"
  }

  fn id(&self) -> &'static str {
    "attribute-target-support"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    for attribute in context.attributes() {
      let attribute_name = &attribute.name.value;

      let matching = BUILTINS
        .iter()
        .filter(|f| {
          matches!(
            f,
            Builtin::Attribute { name, .. } if *name == attribute_name.as_str()
          )
        })
        .collect::<Vec<_>>();

      if matching.is_empty() {
        continue;
      }

      let Some(target_type) = attribute.target else {
        continue;
      };

      let is_valid_target = matching.iter().any(|attr| {
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
