use super::*;

pub struct AttributeTargetSupportRule;

impl Rule for AttributeTargetSupportRule {
  fn id(&self) -> &'static str {
    "attribute-target-support"
  }

  fn display_name(&self) -> &'static str {
    "Attribute Target Support"
  }

  fn run(&self, ctx: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let root = match ctx.tree() {
      Some(tree) => tree.root_node(),
      None => return diagnostics,
    };

    let document = ctx.document();

    for attribute_node in root.find_all("attribute") {
      for identifier_node in attribute_node.find_all("identifier") {
        let attribute_name = document.get_node_text(&identifier_node);

        let matching = builtins::BUILTINS
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

        let Some(parent) = attribute_node.parent() else {
          continue;
        };

        let Some(target_type) = Self::attribute_target_from_kind(parent.kind())
        else {
          continue;
        };

        let is_valid_target = matching.iter().any(|attr| {
          if let Builtin::Attribute { targets, .. } = attr {
            targets
              .iter()
              .any(|target| target.is_valid_for(target_type))
          } else {
            false
          }
        });

        if !is_valid_target {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: attribute_node.get_range(),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!(
              "Attribute `{attribute_name}` cannot be applied to {target_type} target",
            ),
            ..Default::default()
          }));
        }
      }
    }

    diagnostics
  }
}

impl AttributeTargetSupportRule {
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
