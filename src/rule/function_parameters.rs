use super::*;

define_rule! {
  /// Reports invalid user-defined function parameter lists.
  FunctionParametersRule {
    id: "function-parameters",
    message: "invalid function parameters",
    run(context) {
      context
        .functions()
        .iter()
        .flat_map(|function| {
          let mut seen = HashSet::new();

          function
            .parameters
            .iter()
            .filter(move |parameter| !seen.insert(parameter.value.clone()))
            .map(|parameter| {
              Diagnostic::error(
                format!("Duplicate parameter `{}`", parameter.value),
                parameter.range,
              )
            })
        })
        .collect()
    }
  }
}
