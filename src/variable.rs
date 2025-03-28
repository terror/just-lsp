use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Variable {
  pub(crate) name: TextNode,
  pub(crate) export: bool,
  pub(crate) content: String,
  pub(crate) range: lsp::Range,
}

impl Hash for Variable {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name.value.hash(state);
  }
}
