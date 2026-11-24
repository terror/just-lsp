use super::*;

define_rule! {
  UnusedFunctionRule {
    id: "unused-function",
    message: "unused function",
    run(context) {
      let called_functions = context.view().called_functions();

      context
        .view()
        .resolved_functions()
        .filter(|function| {
          function.uri == context.document().uri
            && !function.name.value.starts_with('_')
            && !called_functions.contains(&function.name.value)
        })
        .map(|function| {
          Diagnostic::warning(
            format!("Function `{}` appears unused", function.name.value),
            function.name.range,
          )
        })
        .collect()
    }
  }
}
