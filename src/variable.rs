use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variable {
  pub attributes: Vec<Attribute>,
  pub content: String,
  pub eager: bool,
  pub export: bool,
  pub name: TextNode,
  pub range: lsp::Range,
}
