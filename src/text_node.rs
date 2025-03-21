use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextNode {
  pub(crate) value: String,
  pub(crate) range: lsp::Range,
}

impl TextNode {
  pub(crate) fn is_quoted(&self) -> bool {
    if self.value.len() < 2 {
      return false;
    }

    matches!(
      (
        self.value.chars().next().unwrap(),
        self.value.chars().last().unwrap()
      ),
      ('"', '"') | ('\'', '\'')
    )
  }
}
