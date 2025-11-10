use super::*;

#[derive(Debug, PartialEq)]
pub struct Alias {
  pub name: TextNode,
  pub value: TextNode,
  pub range: lsp::Range,
}
