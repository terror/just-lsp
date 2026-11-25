use super::*;

define_rule! {
  /// Warns when an attribute name isn't part of the known builtin attribute
  /// set.
  UnknownAttributeRule {
    id: "unknown-attribute",
    message: "unknown attribute",
  }
}

impl UnknownAttributeRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for attribute in context.attributes() {
      let attribute_name = &attribute.name.value;

      if Builtin::attribute(attribute_name).is_none() {
        let suggestion = attribute_name
          .find_suggestion(Builtin::attributes().map(Builtin::name));

        let message = match &suggestion {
          Some(suggestion) => format!(
            "Unknown attribute `{attribute_name}`. Did you mean `{suggestion}`?"
          ),
          None => format!("Unknown attribute `{attribute_name}`"),
        };

        let quickfix = suggestion
          .map(|suggestion| Quickfix::replacement(&attribute.name, suggestion));

        diagnostics.push(
          Diagnostic::error(message, attribute.name.range).quickfix(quickfix),
        );
      }
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn attributes_invalid_inline() {
    AnalyzerTest::new(indoc! {
      "
      [group: 'foo', foo]
      foo:
        echo \"foo\"
      "
    })
    .error(
      "Unknown attribute `foo`. Did you mean `doc`?",
      lsp::Range::at(0, 15, 0, 18),
    )
    .run();
  }

  #[test]
  fn attributes_shell_recognized() {
    AnalyzerTest::new(indoc! {
      "
      [shell]
      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_unknown() {
    AnalyzerTest::new(indoc! {
      "
      [unknown_attribute]
      foo:
        echo \"foo\"
      "
    })
    .error(
      "Unknown attribute `unknown_attribute`",
      lsp::Range::at(0, 1, 0, 18),
    )
    .run();
  }
}
