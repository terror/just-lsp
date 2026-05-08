use super::*;

const EXPRESSION_ATTRIBUTES: &[&str] = &["confirm", "env", "working-directory"];

define_rule! {
  AttributeArgumentExpressionsRule {
    id: "attribute-argument-expressions",
    message: "invalid attribute argument expression",
    run(context) {
      let Some(tree) = context.tree() else {
        return Vec::new();
      };

      tree
        .root_node()
        .find_all("attribute")
        .into_iter()
        .flat_map(|attribute| {
          attribute
            .find_all("^identifier")
            .into_iter()
            .flat_map(|identifier| Self::validate(context, identifier))
            .collect::<Vec<_>>()
        })
        .collect()
    }
  }
}

impl AttributeArgumentExpressionsRule {
  fn string_literal_expression(node: Node) -> bool {
    let Some(value) = node.find("^value") else {
      return false;
    };

    let mut cursor = value.walk();

    let children = value.named_children(&mut cursor).collect::<Vec<_>>();

    match children.as_slice() {
      [child] => {
        child.kind() == "string" && child.find("format_string").is_none()
      }
      _ => false,
    }
  }

  fn validate(context: &RuleContext, identifier: Node) -> Vec<Diagnostic> {
    let document = context.document();

    let attribute_name = document.get_node_text(&identifier);

    match context.builtin_attributes(&attribute_name) {
      [] => return Vec::new(),
      _ if EXPRESSION_ATTRIBUTES.contains(&attribute_name.as_str()) => {
        return Vec::new();
      }
      _ => {}
    }

    identifier
      .siblings()
      .take_while(|node| node.kind() != "identifier")
      .filter(|node| node.kind() == "expression")
      .filter(|node| !Self::string_literal_expression(*node))
      .map(|node| {
        Diagnostic::error(
          format!(
            "Attribute `{attribute_name}` arguments must be string literals"
          ),
          node.get_range(document),
        )
      })
      .collect()
  }
}
