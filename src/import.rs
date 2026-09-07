use super::*;

#[derive(Debug, PartialEq)]
pub struct Import {
  pub attributes: Vec<Attribute>,
  pub optional: bool,
  pub path: TextNode,
  pub range: lsp::Range,
}

impl Import {
  #[must_use]
  pub fn is_dynamic(&self) -> bool {
    self.path.value.starts_with(['f', 'x'])
  }

  pub(crate) fn is_enabled(&self) -> bool {
    self
      .attributes
      .iter()
      .filter_map(Attribute::condition)
      .reduce(|left, right| left || right)
      .unwrap_or(true)
  }

  #[must_use]
  pub fn resolve(&self, base_uri: &lsp::Url) -> Option<PathBuf> {
    let raw = self.path.value.trim_matches(|c| c == '\'' || c == '"');

    if raw.is_empty() {
      return None;
    }

    let path = if let Some(rest) = raw.strip_prefix("~/") {
      dirs::home_dir()?.join(rest)
    } else {
      base_uri.file_path().ok()?.parent()?.join(raw)
    };

    Some(path)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, tempfile::Builder};

  fn import(path: &str) -> Import {
    Import {
      attributes: Vec::new(),
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
  fn is_enabled() {
    #[track_caller]
    fn case(attributes: &[&str], expected: bool) {
      let import = Import {
        attributes: attributes
          .iter()
          .map(|name| Attribute {
            name: TextNode {
              value: (*name).into(),
              ..Default::default()
            },
            ..Default::default()
          })
          .collect(),
        ..import("'foo.just'")
      };

      assert_eq!(import.is_enabled(), expected);
    }

    let disabled = if cfg!(windows) { "unix" } else { "windows" };

    case(&[], true);
    case(&["foo"], true);
    case(&[env::consts::OS], true);
    case(&["unix"], cfg!(unix));
    case(&[disabled], false);
    case(&[disabled, "foo"], false);
    case(&[disabled, env::consts::OS], true);
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
