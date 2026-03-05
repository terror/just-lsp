use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Variable {
  pub(crate) content: String,
  pub(crate) export: bool,
  pub(crate) name: TextNode,
  pub(crate) range: lsp::Range,
  pub(crate) unexport: bool,
}
