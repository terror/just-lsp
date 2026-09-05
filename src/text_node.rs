use super::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TextNode {
  pub range: lsp::Range,
  pub value: String,
}

impl TextNode {
  #[must_use]
  pub fn from_node(node: &Node, document: &Document) -> Self {
    Self {
      range: node.get_range(document),
      value: document.get_node_text(node),
    }
  }
}
