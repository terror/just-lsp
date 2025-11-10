use super::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TextNode {
  pub value: String,
  pub range: lsp::Range,
}
