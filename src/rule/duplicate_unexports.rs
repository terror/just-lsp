use super::*;

define_rule! {
  DuplicateUnexportRule {
    id: "duplicate-unexport",
    message: "duplicate unexport",
    run(context) {
      let mut diagnostics = Vec::new();

      let mut seen = HashSet::new();

      for unexport in context.unexports() {
        if !seen.insert(unexport.name.value.clone()) {
          diagnostics.push(Diagnostic::error(
            format!(
              "Variable `{}` is unexported multiple times",
              unexport.name.value
            ),
            unexport.name.range,
          ));
        }
      }

      diagnostics
    }
  }
}
