use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unexport {
  pub attributes: Vec<Attribute>,
  pub name: TextNode,
  pub range: lsp::Range,
}
