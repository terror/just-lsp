use super::*;

const VALID_KWARGS: &[&str] = &["extra", "inputs", "outputs"];

define_rule! {
  CacheAttributeRule {
    id: "cache-attribute",
    message: "invalid cache attribute",
    run(context) {
      let Some(tree) = context.tree() else {
        return Vec::new();
      };

      let document = context.document();
      let mut diagnostics = Vec::new();

      for attribute in tree.root_node().find_all("attribute") {
        for identifier in attribute.find_all("^identifier") {
          if document.get_node_text(&identifier) != "cache" {
            continue;
          }

          let arguments = identifier
            .siblings()
            .take_while(|node| node.kind() != "identifier")
            .filter(|node| node.start_byte() != node.end_byte())
            .collect::<Vec<_>>();

          for argument in arguments.iter().filter(|node| node.kind() == "expression") {
            diagnostics.push(Diagnostic::error(
              "Attribute `cache` only accepts keyword arguments",
              argument.get_range(document),
            ));
          }

          for argument in arguments
            .iter()
            .filter(|node| node.kind() == "attribute_named_param")
          {
            let Some(name) = argument.child_by_field_name("name") else {
              continue;
            };

            let name = document.get_node_text(&name);

            if !VALID_KWARGS.contains(&name.as_str()) {
              diagnostics.push(Diagnostic::error(
                format!(
                  "Unknown `[cache]` keyword `{name}`, expected one of {}",
                  VALID_KWARGS.join(", ")
                ),
                argument.get_range(document),
              ));
            } else if argument.child_by_field_name("value").is_none() {
              diagnostics.push(Diagnostic::error(
                format!("`[cache]` keyword `{name}` requires a value"),
                argument.get_range(document),
              ));
            }
          }
        }
      }

      diagnostics
    }
  }
}
