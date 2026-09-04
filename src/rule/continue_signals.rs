use super::*;

const SIGNALS: &[&str] = &["SIGHUP", "SIGINT", "SIGQUIT"];

define_rule! {
  ContinueSignalsRule {
    id: "continue-signals",
    message: "invalid continue signal",
    run(context) {
      context
        .attributes()
        .iter()
        .filter(|attribute| attribute.name.value == "continue")
        .flat_map(|attribute| &attribute.arguments)
        .filter_map(|argument| {
          let signal = StringLiteral::parse_plain(&argument.value)?.cooked;

          if SIGNALS.contains(&signal.as_str()) {
            None
          } else {
            Some(Diagnostic::error(
              format!(
                "Invalid signal `{signal}`: expected `SIGHUP`, `SIGINT`, or `SIGQUIT`"
              ),
              argument.range,
            ))
          }
        })
        .collect()
    }
  }
}
