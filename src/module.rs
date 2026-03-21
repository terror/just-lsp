use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Module {
  pub(crate) name: TextNode,
  pub(crate) optional: bool,
  pub(crate) path: Option<TextNode>,
  pub(crate) range: lsp::Range,
}

impl Module {
  pub(crate) fn resolve(&self, base_uri: &lsp::Url) -> Option<PathBuf> {
    let base_dir = base_uri.to_file_path().ok()?.parent()?.to_path_buf();

    if let Some(path_node) = &self.path {
      let raw = path_node.value.trim_matches(|c| c == '\'' || c == '"');

      if raw.is_empty() {
        return None;
      }

      return Some(match raw.strip_prefix("~/") {
        Some(rest) => dirs::home_dir()?.join(rest),
        None => base_dir.join(raw),
      });
    }

    let name = &self.name.value;

    [
      base_dir.join(format!("{name}.just")),
      base_dir.join(name).join("mod.just"),
      base_dir.join(name).join("justfile"),
    ]
    .into_iter()
    .find(|path| path.is_file())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn module(name: &str, path: Option<&str>) -> Module {
    Module {
      name: TextNode {
        value: name.to_owned(),
        range: lsp::Range::default(),
      },
      optional: false,
      path: path.map(|p| TextNode {
        value: p.to_owned(),
        range: lsp::Range::default(),
      }),
      range: lsp::Range::default(),
    }
  }

  #[test]
  fn explicit_path() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    let expected = directory.path().join("bar.just");

    assert_eq!(
      module("foo", Some("'bar.just'")).resolve(&base).unwrap(),
      expected,
    );

    assert_eq!(
      module("foo", Some("\"bar.just\"")).resolve(&base).unwrap(),
      expected,
    );
  }

  #[test]
  fn explicit_path_home_directory() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(
      module("foo", Some("'~/bar.just'")).resolve(&base).unwrap(),
      dirs::home_dir().unwrap().join("bar.just"),
    );
  }

  #[test]
  fn empty_explicit_path_returns_none() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(module("foo", Some("''")).resolve(&base), None);
    assert_eq!(module("foo", Some("\"\"")).resolve(&base), None);
  }

  #[test]
  fn implicit_mod_just() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    std::fs::create_dir(directory.path().join("foo")).unwrap();
    std::fs::write(directory.path().join("foo/mod.just"), "").unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(
      module("foo", None).resolve(&base).unwrap(),
      directory.path().join("foo/mod.just"),
    );
  }

  #[test]
  fn implicit_justfile() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    std::fs::create_dir(directory.path().join("foo")).unwrap();
    std::fs::write(directory.path().join("foo/justfile"), "").unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(
      module("foo", None).resolve(&base).unwrap(),
      directory.path().join("foo/justfile"),
    );
  }

  #[test]
  fn implicit_prefers_name_dot_just() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    std::fs::write(directory.path().join("foo.just"), "").unwrap();
    std::fs::create_dir(directory.path().join("foo")).unwrap();
    std::fs::write(directory.path().join("foo/mod.just"), "").unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(
      module("foo", None).resolve(&base).unwrap(),
      directory.path().join("foo.just"),
    );
  }

  #[test]
  fn implicit_no_file_returns_none() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(module("foo", None).resolve(&base), None);
  }
}
