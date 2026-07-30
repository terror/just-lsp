use super::*;

#[derive(Debug, PartialEq)]
pub struct Setting {
  pub attributes: Vec<Attribute>,
  pub kind: SettingKind,
  pub name: TextNode,
  pub range: lsp::Range,
  pub value: TextNode,
}

impl Setting {
  #[must_use]
  pub fn from_node(node: &Node, document: &Document) -> Option<Self> {
    let range = node.get_range(document);

    let name_node = node.child_by_field_name("left")?;

    let name = TextNode {
      value: document.get_node_text(&name_node),
      range: name_node.get_range(document),
    };

    let mut cursor = node.walk();

    let right_children = node
      .children_by_field_name("right", &mut cursor)
      .collect::<Vec<_>>();

    let boolean_child = right_children
      .iter()
      .find(|child| child.kind() == "boolean");

    let expression_child = right_children
      .iter()
      .find(|child| child.kind() == "expression");

    let string_child =
      right_children.iter().find(|child| child.kind() == "string");

    let value = boolean_child
      .or(expression_child)
      .or(string_child)
      .map_or_else(
        || TextNode {
          range: lsp::Range {
            start: name.range.end,
            end: name.range.end,
          },
          value: String::new(),
        },
        |value| TextNode {
          value: document.get_node_text(value),
          range: value.get_range(document),
        },
      );

    let kind = if expression_child.is_some_and(|expression| {
      expression
        .named_child(0)
        .and_then(|value| value.named_child(0))
        .is_some_and(|child| child.kind() == "list_literal")
    }) {
      SettingKind::Array
    } else if let Some(boolean) = boolean_child {
      SettingKind::Boolean(document.get_node_text(boolean) == "true")
    } else if string_child.is_some() || expression_child.is_some() {
      SettingKind::String
    } else if right_children.is_empty() {
      SettingKind::Boolean(true)
    } else {
      return None;
    };

    Some(Setting {
      attributes: document.attributes_for_node(node),
      kind,
      name,
      range,
      value,
    })
  }

  #[must_use]
  pub fn has_attribute(&self, name: &str) -> bool {
    self
      .attributes
      .iter()
      .any(|attribute| attribute.name.value == name)
  }
}
