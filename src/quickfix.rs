use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Quickfix {
  pub edits: Vec<lsp::TextEdit>,
  pub range: lsp::Range,
  pub title: String,
}

impl Quickfix {
  #[must_use]
  pub fn replacement(name: &TextNode, replacement: impl Into<String>) -> Self {
    let replacement = replacement.into();

    Self {
      edits: vec![lsp::TextEdit {
        range: name.range,
        new_text: replacement.clone(),
      }],
      range: name.range,
      title: format!("Replace `{}` with `{replacement}`", name.value),
    }
  }
}
