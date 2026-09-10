use super::*;

define_rule! {
  /// Reports invalid user-defined function parameter lists.
  DuplicateFunctionParameterRule {
    id: "duplicate-function-parameter",
    message: "invalid function parameters",
    run(context) {
      context
        .local_declarations(context.functions())
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
