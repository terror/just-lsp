use super::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct TextNode {
  pub(crate) value: String,
  pub(crate) range: lsp::Range,
}
