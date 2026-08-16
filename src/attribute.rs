use super::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Attribute {
  pub arguments: Vec<TextNode>,
  pub name: TextNode,
  pub range: lsp::Range,
  pub target: Option<AttributeTarget>,
}
