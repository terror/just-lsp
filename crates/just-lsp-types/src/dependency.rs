use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Dependency {
  pub name: String,
  pub arguments: Vec<TextNode>,
  pub range: lsp::Range,
}
