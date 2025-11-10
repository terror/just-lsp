use super::*;

#[derive(Debug, PartialEq, Eq)]
pub struct FunctionCall {
  pub name: TextNode,
  pub arguments: Vec<TextNode>,
  pub range: lsp::Range,
}
