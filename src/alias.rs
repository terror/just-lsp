use super::*;

#[derive(Debug, PartialEq)]
pub struct Alias {
  pub attributes: Vec<Attribute>,
  pub name: TextNode,
  pub range: lsp::Range,
  pub value: TextNode,
}
