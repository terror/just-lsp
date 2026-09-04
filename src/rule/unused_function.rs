use super::*;

define_rule! {
  /// Finds user-defined functions that are never called in the document or
  /// any of its imports.
  UnusedFunctionRule {
    id: "unused-function",
    message: "unused function",
    run(context) {
      // Name-only matching deliberately treats all same-named definitions as
      // used when exact resolution is ambiguous, such as across OS groups.
      let called = once(context.document())
        .chain(context.imported_documents())
        .flat_map(Document::function_calls)
        .map(|function_call| function_call.name.value)
        .collect::<HashSet<_>>();

      context
        .document()
        .functions()
        .into_iter()
        .filter(|function| {
          !function.name.value.starts_with('_')
            && !called.contains(&function.name.value)
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
