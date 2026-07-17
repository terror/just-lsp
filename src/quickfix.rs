use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Quickfix {
  pub edits: Vec<lsp::TextEdit>,
  pub range: lsp::Range,
  pub title: String,
}

impl Quickfix {
  #[must_use]
  pub fn removal(range: lsp::Range, title: impl Into<String>) -> Self {
    Self {
      edits: vec![lsp::TextEdit {
        range,
        new_text: String::new(),
      }],
      range,
      title: title.into(),
    }
  }

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

  #[must_use]
  pub fn setting_attribute(
    setting: &Setting,
    document: &Document,
    attribute: &str,
    replacement: &str,
  ) -> Self {
    let line = document
      .content
      .line(setting.range.start.line as usize)
      .to_string();

    let line = line.replacen(&setting.name.value, replacement, 1);

    Self {
      edits: vec![lsp::TextEdit {
        range: setting.range,
        new_text: format!("[{attribute}]\n{line}"),
      }],
      range: setting.name.range,
      title: format!(
        "Replace `{}` with `[{attribute}] set {replacement}`",
        setting.name.value
      ),
    }
  }
}
