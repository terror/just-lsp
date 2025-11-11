use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Dependency {
  pub arguments: Vec<TextNode>,
  pub name: String,
  pub range: lsp::Range,
}
