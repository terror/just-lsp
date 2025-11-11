use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Dependency {
  pub(crate) arguments: Vec<TextNode>,
  pub(crate) name: String,
  pub(crate) range: lsp::Range,
}
