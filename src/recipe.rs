use super::*;

#[derive(Debug, PartialEq)]
#[allow(unused)]
pub(crate) struct Recipe {
  pub(crate) name: String,
  pub(crate) dependencies: Vec<String>,
  pub(crate) content: String,
  pub(crate) range: lsp::Range,
}
