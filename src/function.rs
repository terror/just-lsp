use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
  pub attributes: Vec<Attribute>,
  pub body: String,
  pub content: String,
  pub name: TextNode,
  pub parameters: Vec<TextNode>,
  pub range: lsp::Range,
}
