use super::*;

define_rule! {
  UnusedFunctionRule {
    id: "unused-function",
    message: "unused function",
    run(context) {
      let mut functions = context.view().resolved_functions();

      for function_call in context.view().documents().flat_map(Document::function_calls) {
        functions.remove(&function_call.name.value);
      }

      functions
        .into_values()
        .filter(|function| {
          function.uri == context.document().uri
            && !function.name.value.starts_with('_')
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
