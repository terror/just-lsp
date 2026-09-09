use super::*;

define_rule! {
  DuplicateUnexportRule {
    id: "duplicate-unexport",
    message: "duplicate unexport",
    run(context) {
      let mut diagnostics = Vec::new();

      for unexport in context.duplicate_declarations(
        context.unexports(),
        |unexport| &unexport.name,
        |unexport| &unexport.attributes,
      ) {
        diagnostics.push(Diagnostic::error(
          format!(
            "Variable `{}` is unexported multiple times",
            unexport.name.value
          ),
          unexport.name.range,
        ));
      }

      diagnostics
    }
  }
}
