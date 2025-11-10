use super::*;

/// Warns when an attribute name isn’t part of the known builtin attribute set.
pub struct UnknownAttributeRule;

impl Rule for UnknownAttributeRule {
  fn id(&self) -> &'static str {
    "unknown-attribute"
  }

  fn display_name(&self) -> &'static str {
    "Unknown Attribute"
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
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: identifier_node.get_range(),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!("Unknown attribute `{attribute_name}`"),
            ..Default::default()
          }));
        }
      }
    }

    diagnostics
  }
}
