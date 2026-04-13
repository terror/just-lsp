use super::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TextNode {
  pub range: lsp::Range,
  pub value: String,
}
