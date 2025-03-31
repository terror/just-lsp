use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Variable {
  pub(crate) name: TextNode,
  pub(crate) export: bool,
  pub(crate) content: String,
  pub(crate) range: lsp::Range,
}
