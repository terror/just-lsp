use super::*;

pub struct AttributesRule;

impl Rule for AttributesRule {
  fn id(&self) -> &'static str {
    "attributes"
  }

  fn display_name(&self) -> &'static str {
    "Attributes"
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

        let matching_attributes: Vec<_> = builtins::BUILTINS
          .iter()
          .filter(|f| matches!(f, Builtin::Attribute { name, .. } if *name == attribute_name))
          .collect();

        if matching_attributes.is_empty() {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: identifier_node.get_range(),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!("Unknown attribute `{attribute_name}`"),
            ..Default::default()
          }));

          continue;
        }

        let argument_count = identifier_node
          .find_siblings_until("string", "identifier")
          .len();

        let has_arguments = argument_count > 0;

        let parameter_mismatch = matching_attributes.iter().all(|attr| {
          if let Builtin::Attribute { parameters, .. } = attr {
            (parameters.is_some() && !has_arguments)
              || (parameters.is_none() && has_arguments)
              || (parameters.map_or(0, |_| 1) < argument_count)
          } else {
            false
          }
        });

        if parameter_mismatch {
          let required_argument_count = matching_attributes
            .iter()
            .find_map(|attr| {
              if let Builtin::Attribute { parameters, .. } = attr {
                parameters.map(|_| 1)
              } else {
                None
              }
            })
            .unwrap_or(0);

          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: attribute_node.get_range(),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!(
              "Attribute `{attribute_name}` got {argument_count} {} but takes {required_argument_count} {}",
              Count("argument", argument_count),
              Count("argument", required_argument_count),
            ),
            ..Default::default()
          }));
        }

        if let Some(parent) = attribute_node.parent() {
          let target_type = match parent.kind() {
            "alias" => AttributeTarget::Alias,
            "assignment" => AttributeTarget::Assignment,
            "export" => AttributeTarget::Assignment,
            "module" => AttributeTarget::Module,
            "recipe" => AttributeTarget::Recipe,
            _ => {
              diagnostics.push(self.diagnostic(lsp::Diagnostic {
                range: attribute_node.get_range(),
                severity: Some(lsp::DiagnosticSeverity::ERROR),
                message: format!(
                  "Attribute `{attribute_name}` applied to invalid target",
                ),
                ..Default::default()
              }));

              continue;
            }
          };

          let is_valid_target = matching_attributes
            .iter()
            .filter_map(|attr| {
              if let Builtin::Attribute { targets, .. } = attr {
                Some(targets)
              } else {
                None
              }
            })
            .any(|targets| {
              targets
                .iter()
                .any(|target| target.is_valid_for(target_type))
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
    }

    diagnostics
  }
}
