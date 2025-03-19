use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Alias {
  pub(crate) left: String,
  pub(crate) right: String,
  pub(crate) range: lsp::Range,
}
