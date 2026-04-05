use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Function {
  pub(crate) body: String,
  pub(crate) content: String,
  pub(crate) name: TextNode,
  pub(crate) parameters: Vec<TextNode>,
  pub(crate) range: lsp::Range,
}
