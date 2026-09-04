use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Quickfix {
  edits: Vec<lsp::TextEdit>,
  title: String,
}

impl Quickfix {
  #[must_use]
  pub fn edit(
    title: impl Into<String>,
    range: lsp::Range,
    new_text: impl Into<String>,
  ) -> Self {
    Self::new(
      title,
      [lsp::TextEdit {
        range,
        new_text: new_text.into(),
      }],
    )
  }

  #[must_use]
  pub fn edits(&self) -> &[lsp::TextEdit] {
    &self.edits
  }

  #[must_use]
  pub fn new(
    title: impl Into<String>,
    edits: impl IntoIterator<Item = lsp::TextEdit>,
  ) -> Self {
    Self {
      edits: edits.into_iter().collect(),
      title: title.into(),
    }
  }

  #[must_use]
  pub fn removal(range: lsp::Range, title: impl Into<String>) -> Self {
    Self::edit(title, range, String::new())
  }

  #[must_use]
  pub fn replacement(name: &TextNode, replacement: impl Into<String>) -> Self {
    let replacement = replacement.into();

    Self::edit(
      format!("Replace `{}` with `{replacement}`", name.value),
      name.range,
      replacement,
    )
  }

  #[must_use]
  pub fn title(&self) -> &str {
    &self.title
  }
}
