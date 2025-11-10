use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Attribute {
  pub name: TextNode,
  pub arguments: Vec<TextNode>,
  pub target: Option<AttributeTarget>,
  pub range: lsp::Range,
}
