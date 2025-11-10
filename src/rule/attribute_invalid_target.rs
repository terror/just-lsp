use super::*;

/// Ensures attributes only appear on syntax nodes that actually accept attributes.
pub struct AttributeInvalidTargetRule;

impl Rule for AttributeInvalidTargetRule {
  fn id(&self) -> &'static str {
    "attribute-invalid-target"
  }

  fn display_name(&self) -> &'static str {
    "Attribute Invalid Target"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let root = match context.tree() {
      Some(tree) => tree.root_node(),
      None => return diagnostics,
    };

    let document = context.document();

    for attribute_node in root.find_all("attribute") {
      for identifier_node in attribute_node.find_all("identifier") {
        let attribute_name = document.get_node_text(&identifier_node);

        let is_known = builtins::BUILTINS.iter().any(|f| {
          matches!(
            f,
            Builtin::Attribute { name, .. } if *name == attribute_name.as_str()
          )
        });

        if !is_known {
          continue;
        }

        let Some(parent) = attribute_node.parent() else {
          continue;
        };

        if Self::attribute_target_from_kind(parent.kind()).is_none() {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: attribute_node.get_range(),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!(
              "Attribute `{attribute_name}` applied to invalid target",
            ),
            ..Default::default()
          }));
        }
      }
    }

    diagnostics
  }
}

impl AttributeInvalidTargetRule {
  fn attribute_target_from_kind(kind: &str) -> Option<AttributeTarget> {
    match kind {
      "alias" => Some(AttributeTarget::Alias),
      "assignment" | "export" => Some(AttributeTarget::Assignment),
      "module" => Some(AttributeTarget::Module),
      "recipe" => Some(AttributeTarget::Recipe),
      _ => None,
    }
  }
}
