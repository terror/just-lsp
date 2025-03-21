use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Variable {
  pub(crate) name: TextNode,
  pub(crate) content: String,
  pub(crate) range: lsp::Range,
}
