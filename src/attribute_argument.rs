use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeArgument {
  Keyword {
    name: TextNode,
    value: Option<AttributeExpression>,
    range: lsp::Range,
  },
  Positional(AttributeExpression),
}

impl AttributeArgument {
  pub(crate) fn from_node(
    node: &Node,
    document: &Document,
    keywords: bool,
  ) -> Option<Self> {
    if node.start_byte() == node.end_byte() {
      return None;
    }

    let value = node.find("^value");

    let name = if node.kind() == "attribute_named_param" {
      node.child_by_field_name("name")
    } else {
      value.as_ref().and_then(|value| value.find("^identifier"))
    };

    if let Some(name) = name
      && name.byte_range() == node.byte_range()
    {
      return Some(if keywords {
        Self::Keyword {
          name: TextNode::from_node(&name, document),
          value: None,
          range: document.get_range(node),
        }
      } else {
        Self::Positional(AttributeExpression::from_node(&name, document))
      });
    }

    match node.kind() {
      "attribute_named_param" => Some(Self::Keyword {
        name: TextNode::from_node(&node.child_by_field_name("name")?, document),
        value: node
          .child_by_field_name("value")
          .filter(|value| value.start_byte() != value.end_byte())
          .map(|value| AttributeExpression::from_node(&value, document)),
        range: document.get_range(node),
      }),
      "expression" | "string" => Some(Self::Positional(
        AttributeExpression::from_node(node, document),
      )),
      _ => None,
    }
  }
}
