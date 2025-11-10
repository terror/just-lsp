use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Attribute {
  pub name: TextNode,
  pub arguments: Vec<TextNode>,
  pub range: lsp::Range,
}
