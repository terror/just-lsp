use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Attribute {
  pub(crate) name: TextNode,
  pub(crate) arguments: Vec<TextNode>,
  pub(crate) range: lsp::Range,
}
