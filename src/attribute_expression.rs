use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeExpression {
  pub kind: AttributeExpressionKind,
  pub text: TextNode,
}

impl AttributeExpression {
  pub(crate) fn from_node(node: &Node, document: &Document) -> Self {
    let value = node.find("^value");

    let string = if node.kind() == "string" {
      Some(*node)
    } else {
      value.as_ref().and_then(|value| value.find("^string"))
    };

    let kind = if let Some(string) = string
      && string.byte_range() == node.byte_range()
      && string.find("format_string").is_none()
    {
      AttributeExpressionKind::StringLiteral
    } else if node.find("function_call, external_command").is_some() {
      AttributeExpressionKind::Any
    } else {
      AttributeExpressionKind::Const
    };

    Self {
      kind,
      text: TextNode {
        range: document.get_range(node),
        value: document.get_node_text(node),
      },
    }
  }
}
