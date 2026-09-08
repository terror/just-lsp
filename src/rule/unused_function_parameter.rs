use super::*;

define_rule! {
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
        .find_all("function_parameters > identifier")
        .into_iter()
        .filter(|parameter| !parameter.is_missing())
        .filter_map(|parameter| {
          let body = parameter
            .get_parent("function_definition")?
            .child_by_field_name("body")
            .filter(|body| !body.is_missing())?;

          let name = document.get_node_text(&parameter);

          let used = body
            .find_all("value > identifier")
            .iter()
            .any(|identifier| document.get_node_text(identifier) == name);

          (!name.starts_with('_') && !used).then(|| {
            Diagnostic::warning(
              format!("Function parameter `{name}` appears unused"),
              parameter.get_range(document),
            )
          })
        })
        .collect()
    }
  }
}
