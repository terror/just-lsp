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

    let path = if let Some(rest) = raw.strip_prefix("~/") {
      dirs::home_dir()?.join(rest)
    } else {
      base_uri.to_file_path().ok()?.parent()?.join(raw)
    };

    Some(path)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn import(path: &str) -> Import {
    Import {
      optional: false,
      path: TextNode {
        value: path.to_owned(),
        range: lsp::Range::default(),
      },
      range: lsp::Range::default(),
    }
  }

  #[test]
  fn empty_path_returns_none() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(import("''").resolve(&base), None);
    assert_eq!(import("\"\"").resolve(&base), None);
    assert_eq!(import("").resolve(&base), None);
  }

  #[test]
  fn home_directory() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(
      import("'~/bar.just'").resolve(&base).unwrap(),
      dirs::home_dir().unwrap().join("bar.just"),
    );
  }

  #[test]
  fn resolve() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(
      import("'bar.just'").resolve(&base).unwrap(),
      directory.path().join("bar.just")
    );

    assert_eq!(
      import("\"bar.just\"").resolve(&base).unwrap(),
      directory.path().join("bar.just")
    );

    assert_eq!(
      import("bar.just").resolve(&base).unwrap(),
      directory.path().join("bar.just")
    );

    assert_eq!(
      import("'sub/bar.just'").resolve(&base).unwrap(),
      directory.path().join("sub/bar.just")
    );
  }
}
