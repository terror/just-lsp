use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Alias {
  pub(crate) name: TextNode,
  pub(crate) range: lsp::Range,
  pub(crate) value: TextNode,
}
