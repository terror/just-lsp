use super::*;

/// Reports attribute invocations whose argument counts don’t match their builtin
/// definitions.
pub(crate) struct AttributeArgumentsRule;

impl Rule for AttributeArgumentsRule {
  fn display_name(&self) -> &'static str {
    "Attribute Arguments"
  }

  fn id(&self) -> &'static str {
    "attribute-arguments"
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

      let argument_count = attribute.arguments.len();
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
          range: attribute.range,
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

    diagnostics
  }
}
