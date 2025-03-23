use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextNode {
  pub(crate) value: String,
  pub(crate) range: lsp::Range,
}
