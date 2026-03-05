use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Attribute {
  pub(crate) arguments: Vec<TextNode>,
  pub(crate) name: TextNode,
  pub(crate) range: lsp::Range,
  pub(crate) target: Option<AttributeTarget>,
}
