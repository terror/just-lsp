use super::*;

const SIGNALS: &[&str] = &["SIGHUP", "SIGINT", "SIGQUIT"];

define_rule! {
  InvalidContinueSignalRule {
    id: "invalid-continue-signal",
    title: "invalid continue signal",
    severity: Error,
  }
}

impl InvalidContinueSignalRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    context
      .attributes()
      .iter()
      .filter(|attribute| attribute.name.value == "continue")
      .flat_map(Attribute::positional_arguments)
      .filter_map(|argument| {
        let signal = argument.text.value.literal()?;

        if SIGNALS.contains(&signal.as_str()) {
          None
        } else {
          Some(Diagnostic::new(
            format!(
              "Invalid signal `{signal}`: expected `SIGHUP`, `SIGINT`, or `SIGQUIT`"
            ),
            argument.text.range,
          ))
        }
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn continue_attribute_accepts_multiple_arguments() {
    AnalyzerTest::new(indoc! {
      "
      [continue(\"SIGHUP\", \"SIGINT\", \"SIGQUIT\")]
      foo:
        echo foo
      "
    })
    .run();
  }

  #[test]
  fn continue_attribute_accepts_one_argument() {
    AnalyzerTest::new(indoc! {
      "
      [continue(\"SIGINT\")]
      foo:
        echo foo
      "
    })
    .run();
  }

  #[test]
  fn continue_attribute_accepts_zero_arguments() {
    AnalyzerTest::new(indoc! {
      "
      [continue]
      foo:
        echo foo
      "
    })
    .run();
  }

  #[test]
  fn continue_attribute_rejects_unknown_signals() {
    AnalyzerTest::new(indoc! {
      "
      [continue(\"SIGTERM\", \"sigint\")]
      foo:
        echo foo
      "
    })
    .error(
      "Invalid signal `SIGTERM`: expected `SIGHUP`, `SIGINT`, or `SIGQUIT`",
      lsp::Range::at(0, 10, 0, 19),
    )
    .error(
      "Invalid signal `sigint`: expected `SIGHUP`, `SIGINT`, or `SIGQUIT`",
      lsp::Range::at(0, 21, 0, 29),
    )
    .run();
  }
}
