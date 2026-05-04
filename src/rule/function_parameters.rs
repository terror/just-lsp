use super::*;

define_rule! {
  /// Reports invalid user-defined function parameter lists.
  FunctionParametersRule {
    id: "function-parameters",
    message: "invalid function parameters",
    run(context) {
      let mut diagnostics = Vec::new();

      for function in context.functions() {
        let mut seen = HashSet::new();

        for parameter in &function.parameters {
          if !seen.insert(parameter.value.clone()) {
            diagnostics.push(Diagnostic::error(
              format!("Duplicate parameter `{}`", parameter.value),
              parameter.range,
            ));
          }
        }
      }

      diagnostics
    }
  }
}
