use super::*;

define_rule! {
  /// Ensures attributes only appear on syntax nodes that actually accept
  /// attributes.
  InvalidAttributePlacementRule {
    id: "invalid-attribute-placement",
    title: "invalid attribute target",
    severity: Error,
  }
}

impl InvalidAttributePlacementRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for attribute in context.local().attributes() {
      let attribute_name = &attribute.name.value;

      if Builtin::attribute(attribute_name).is_none() {
        continue;
      }

      if attribute.target.is_none() {
        diagnostics.push(Diagnostic::new(
          format!("Attribute `{attribute_name}` applied to invalid target"),
          attribute.range,
        ));
      }
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn attributes_invalid_target() {
    AnalyzerTest::new(indoc! {
      "
      [private]
      "
    })
    .error(
      "Attribute `private` applied to invalid target",
      lsp::Range::at(0, 0, 1, 0),
    )
    .error("Syntax error near `[private]`", lsp::Range::at(0, 0, 1, 0))
    .run();
  }
}
