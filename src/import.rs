use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Import {
  pub(crate) optional: bool,
  pub(crate) path: TextNode,
  pub(crate) range: lsp::Range,
}
