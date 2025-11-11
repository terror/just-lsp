use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FunctionCall {
  pub(crate) arguments: Vec<TextNode>,
  pub(crate) name: TextNode,
  pub(crate) range: lsp::Range,
}
