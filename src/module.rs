use super::*;

#[derive(Debug, PartialEq)]
pub struct Module {
  pub attributes: Vec<Attribute>,
  pub name: TextNode,
  pub optional: bool,
  pub path: Option<TextNode>,
  pub range: lsp::Range,
}

impl Module {
  const DIRECTORY_CANDIDATES: [(&str, bool); 3] = [
    ("mod.just", true),
    ("justfile", false),
    (".justfile", false),
  ];

  fn find_file(
    directory: &Path,
    candidates: &[(&str, bool)],
  ) -> Option<PathBuf> {
    let entries = fs::read_dir(directory)
      .ok()?
      .filter_map(std::result::Result::ok)
      .collect::<Vec<_>>();

    candidates.iter().find_map(|(candidate, case_sensitive)| {
      entries.iter().find_map(|entry| {
        let path = entry.path();

        let name = entry.file_name();

        let name = name.to_str()?;

        let matches = if *case_sensitive {
          name == *candidate
        } else {
          name.eq_ignore_ascii_case(candidate)
        };

        if path.is_file() && matches {
          Some(path)
        } else {
          None
        }
      })
    })
  }

  #[must_use]
  pub fn resolve(&self, base_uri: &lsp::Url) -> Option<PathBuf> {
    let base_dir = base_uri.file_path().ok()?.parent()?.to_path_buf();

    if let Some(path_node) = &self.path {
      let path =
        StringLiteral::resolve_path(&path_node.value, base_uri).ok()??;

      return if path.is_dir() {
        Self::find_file(&path, &Self::DIRECTORY_CANDIDATES)
      } else {
        Some(path)
      };
    }

    let name = &self.name.value;

    let name_just = format!("{name}.just");

    if let Some(name_just) = Self::find_file(&base_dir, &[(&name_just, true)]) {
      return Some(name_just);
    }

    Self::find_file(&base_dir.join(name), &Self::DIRECTORY_CANDIDATES)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, tempfile::Builder};

  fn module(name: &str, path: Option<&str>) -> Module {
    Module {
      attributes: Vec::new(),
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
  fn empty_explicit_path_returns_none() {
    #[track_caller]
    fn case(source: &str) {
      let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

      let base =
        lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

      assert_eq!(module("foo", Some(source)).resolve(&base), None);
    }

    case("''");
    case("\"\"");
    case("x''");
    case("'''  '''");
  }

  #[test]
  fn explicit_directory() {
    #[track_caller]
    fn case(source: &str, file: &str) {
      let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

      fs::create_dir(directory.path().join("foo")).unwrap();
      fs::write(directory.path().join("foo").join(file), "").unwrap();

      let base =
        lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

      assert_eq!(
        module("foo", Some(source)).resolve(&base).unwrap(),
        directory.path().join("foo").join(file),
      );
    }

    case("'foo'", "mod.just");
    case("'foo'", "JUSTFILE");
    case("'foo'", ".JUSTFILE");
    case("x'foo'", "mod.just");
    case(r#""\u{66}oo""#, "mod.just");
  }

  #[test]
  fn explicit_path() {
    #[track_caller]
    fn case(source: &str, expected: &str) {
      let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

      let base =
        lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

      assert_eq!(
        module("foo", Some(source)).resolve(&base),
        Some(directory.path().join(expected)),
      );
    }

    case("'foo.just'", "foo.just");
    case("\"foo.just\"", "foo.just");
    case("x'foo.just'", "foo.just");
    case("x\"foo.just\"", "foo.just");
    case("x'''foo.just'''", "foo.just");
    case("x\"\"\"foo.just\"\"\"", "foo.just");
    case("'''  foo.just'''", "foo.just");
    case(r#""\u{66}oo.just""#, "foo.just");
    case(r"'foo\nbar.just'", r"foo\nbar.just");
    case(r#""foo\nbar.just""#, "foo\nbar.just");
    case(r#""'foo.just'""#, "'foo.just'");
    case(r#"'"foo.just"'"#, "\"foo.just\"");
    case(r#""foo\".just""#, "foo\".just");
    case("x'$$foo.just'", "$foo.just");
  }

  #[test]
  fn explicit_path_home_directory() {
    #[track_caller]
    fn case(source: &str) {
      let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

      let base =
        lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

      assert_eq!(
        module("foo", Some(source)).resolve(&base).unwrap(),
        env::home_dir().unwrap().join("bar.just"),
      );
    }

    case("'~/bar.just'");
    case("x'~/bar.just'");
  }

  #[test]
  fn explicit_path_invalid_literal_returns_none() {
    #[track_caller]
    fn case(source: &str) {
      let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

      let base =
        lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

      assert_eq!(module("foo", Some(source)).resolve(&base), None);
    }

    case("");
    case("'foo");
    case("f'foo'");
    case(r#""\q""#);
  }

  #[test]
  fn implicit_case_insensitive_justfile() {
    #[track_caller]
    fn case(file: &str) {
      let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

      fs::create_dir(directory.path().join("foo")).unwrap();
      fs::write(directory.path().join("foo").join(file), "").unwrap();

      let base =
        lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

      assert_eq!(
        module("foo", None).resolve(&base).unwrap(),
        directory.path().join("foo").join(file),
      );
    }

    case("JUSTFILE");
    case(".JUSTFILE");
  }

  #[test]
  fn implicit_dot_justfile() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::create_dir(directory.path().join("foo")).unwrap();
    fs::write(directory.path().join("foo/.justfile"), "").unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(
      module("foo", None).resolve(&base).unwrap(),
      directory.path().join("foo/.justfile"),
    );
  }

  #[test]
  fn implicit_just_paths_are_case_sensitive() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::write(directory.path().join("FOO.just"), "").unwrap();
    fs::create_dir(directory.path().join("foo")).unwrap();
    fs::write(directory.path().join("foo/MOD.just"), "").unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(module("foo", None).resolve(&base), None);
  }

  #[test]
  fn implicit_justfile() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::create_dir(directory.path().join("foo")).unwrap();
    fs::write(directory.path().join("foo/justfile"), "").unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(
      module("foo", None).resolve(&base).unwrap(),
      directory.path().join("foo/justfile"),
    );
  }

  #[test]
  fn implicit_mod_just() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::create_dir(directory.path().join("foo")).unwrap();
    fs::write(directory.path().join("foo/mod.just"), "").unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(
      module("foo", None).resolve(&base).unwrap(),
      directory.path().join("foo/mod.just"),
    );
  }

  #[test]
  fn implicit_no_file_returns_none() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(module("foo", None).resolve(&base), None);
  }

  #[test]
  fn implicit_prefers_name_dot_just() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::write(directory.path().join("foo.just"), "").unwrap();
    fs::create_dir(directory.path().join("foo")).unwrap();
    fs::write(directory.path().join("foo/mod.just"), "").unwrap();
    fs::write(directory.path().join("foo/.justfile"), "").unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(
      module("foo", None).resolve(&base).unwrap(),
      directory.path().join("foo.just"),
    );
  }
}
