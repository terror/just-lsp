use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Variable {
  pub(crate) name: String,
  pub(crate) range: lsp::Range,
}
