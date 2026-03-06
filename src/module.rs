use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Module {
  pub(crate) name: TextNode,
  pub(crate) optional: bool,
  pub(crate) path: Option<TextNode>,
  pub(crate) range: lsp::Range,
}
