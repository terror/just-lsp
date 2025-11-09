use super::*;

/// Reports attribute invocations whose argument counts don’t match their builtin
/// definitions.
pub struct AttributeArgumentsRule;

impl Rule for AttributeArgumentsRule {
  fn id(&self) -> &'static str {
    "attribute-arguments"
  }

  fn display_name(&self) -> &'static str {
    "Attribute Arguments"
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

        let argument_count = identifier_node
          .find_siblings_until("string", "identifier")
          .len();

        let has_arguments = argument_count > 0;

        let parameter_mismatch = matching.iter().all(|attr| {
          if let Builtin::Attribute { parameters, .. } = attr {
            (parameters.is_some() && !has_arguments)
              || (parameters.is_none() && has_arguments)
              || (parameters.map_or(0, |_| 1) < argument_count)
          } else {
            false
          }
        });

        if parameter_mismatch {
          let required_argument_count = matching
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
      }
    }

    diagnostics
  }
}
