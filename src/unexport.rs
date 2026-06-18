use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unexport {
  pub name: TextNode,
  pub range: lsp::Range,
}
