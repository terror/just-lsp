use super::*;

#[derive(Debug, PartialEq, Eq)]
pub struct FunctionCall {
  pub arguments: Vec<TextNode>,
  pub name: TextNode,
  pub range: lsp::Range,
}
