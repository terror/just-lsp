use super::*;

define_rule! {
  /// Highlights user-defined function parameters that are never referenced in
  /// the function body.
  UnusedFunctionParameterRule {
    id: "unused-function-parameter",
    message: "unused function parameter",
    run(context) {
      let Some(tree) = context.tree() else {
        return Vec::new();
      };

      let document = context.document();

      tree
        .root_node()
        .find_all("function_definition")
        .into_iter()
        .flat_map(|function_node| {
          let Some(body_node) = function_node
            .child_by_field_name("body")
            .filter(|body_node| !body_node.is_missing())
          else {
            return Vec::new();
          };

          let used = body_node
            .find_all("value > identifier")
            .into_iter()
            .map(|identifier_node| document.get_node_text(&identifier_node))
            .collect::<HashSet<_>>();

          let Some(parameters_node) =
            function_node.child_by_field_name("parameters")
          else {
            return Vec::new();
          };

          parameters_node
            .find_all("^identifier")
            .into_iter()
            .filter_map(move |parameter_node| {
              if parameter_node.is_missing() {
                return None;
              }

              let name = document.get_node_text(&parameter_node);

              (!name.starts_with('_') && !used.contains(&name)).then(|| {
                Diagnostic::warning(
                  format!("Function parameter `{name}` appears unused"),
                  parameter_node.get_range(document),
                )
              })
            })
            .collect::<Vec<_>>()
        })
        .collect()
    }
  }
}
