use super::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Variable {
  pub name: TextNode,
  pub export: bool,
  pub content: String,
  pub range: lsp::Range,
}
