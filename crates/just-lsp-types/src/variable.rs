use super::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Variable {
  pub content: String,
  pub export: bool,
  pub name: TextNode,
  pub range: lsp::Range,
}
