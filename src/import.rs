use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Import {
  pub(crate) optional: bool,
  pub(crate) path: TextNode,
  pub(crate) range: lsp::Range,
}

impl Import {
  pub(crate) fn resolve(&self, base_uri: &lsp::Url) -> Option<PathBuf> {
    let raw = self.path.value.trim_matches(|c| c == '\'' || c == '"');

    if raw.is_empty() {
      return None;
    }

    let base_path = base_uri.to_file_path().ok()?;
    let base_dir = base_path.parent()?;

    Some(base_dir.join(raw))
  }
}
