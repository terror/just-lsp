use super::*;

#[derive(Debug, PartialEq)]
pub struct ModuleDeclaration {
  pub attributes: Vec<Attribute>,
  pub name: TextNode,
  pub optional: bool,
  pub path: Option<TextNode>,
  pub range: lsp::Range,
}

impl ModuleDeclaration {
  const DIRECTORY_CANDIDATES: [(&str, bool); 3] = [
    ("mod.just", true),
    ("justfile", false),
    (".justfile", false),
  ];

  pub(crate) fn candidates(&self, base_uri: &lsp::Url) -> Vec<(PathBuf, bool)> {
    let (file, directory) = if let Some(path_node) = &self.path {
      let Ok(Some(path)) =
        StringLiteral::resolve_path(&path_node.value, base_uri)
      else {
        return Vec::new();
      };

      (path.clone(), path)
    } else {
      let Some(base) = base_uri
        .file_path()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
      else {
        return Vec::new();
      };

      (
        base.join(format!("{}.just", self.name.value)),
        base.join(&self.name.value),
      )
    };

    once((file, true))
      .chain(
        Self::DIRECTORY_CANDIDATES
          .iter()
          .map(|(name, case_sensitive)| {
            (directory.join(name), *case_sensitive)
          }),
      )
      .collect()
  }

  pub(crate) fn files(
    &self,
    base_uri: &lsp::Url,
    filesystem: &dyn Filesystem,
  ) -> Vec<PathBuf> {
    let mut candidates = self.candidates(base_uri).into_iter();

    if self.path.is_some() {
      let Some((path, _)) = candidates.next() else {
        return Vec::new();
      };

      if filesystem.is_file(&path) {
        return vec![path];
      }
    }

    candidates
      .flat_map(|(candidate, case_sensitive)| {
        Self::find_files(filesystem, &candidate, case_sensitive)
      })
      .collect()
  }

  fn find_files(
    filesystem: &dyn Filesystem,
    candidate: &Path,
    case_sensitive: bool,
  ) -> Vec<PathBuf> {
    let (Some(directory), Some(candidate_name)) = (
      candidate.parent(),
      candidate.file_name().and_then(|name| name.to_str()),
    ) else {
      return Vec::new();
    };

    filesystem
      .read_dir(directory)
      .unwrap_or_default()
      .into_iter()
      .filter(|path| {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
          return false;
        };

        let matches = if case_sensitive {
          name == candidate_name
        } else {
          name.eq_ignore_ascii_case(candidate_name)
        };

        matches && filesystem.is_file(path)
      })
      .collect()
  }

  pub(crate) fn is_dynamic(&self) -> bool {
    self
      .path
      .as_ref()
      .is_some_and(|path| path.value.starts_with('f'))
  }

  pub(crate) fn location(&self) -> lsp::Range {
    self
      .path
      .as_ref()
      .map_or(self.name.range, |path| path.range)
  }

  pub(crate) fn resolve(
    &self,
    base_uri: &lsp::Url,
    filesystem: &dyn Filesystem,
  ) -> Option<PathBuf> {
    self.files(base_uri, filesystem).into_iter().next()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, tempfile::Builder};

  fn module(name: &str, path: Option<&str>) -> ModuleDeclaration {
    ModuleDeclaration {
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

      assert_eq!(
        module("foo", Some(source)).resolve(&base, &OsFilesystem),
        None
      );
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
        module("foo", Some(source))
          .resolve(&base, &OsFilesystem)
          .unwrap(),
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

      let path = directory.path().join(expected);

      assert_eq!(
        module("foo", Some(source)).resolve(
          &base,
          &MemoryFilesystem::new([(path.clone(), String::new())]),
        ),
        Some(path),
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

      let path = env::home_dir().unwrap().join("bar.just");

      assert_eq!(
        module("foo", Some(source)).resolve(
          &base,
          &MemoryFilesystem::new([(path.clone(), String::new())]),
        ),
        Some(path),
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

      assert_eq!(
        module("foo", Some(source)).resolve(&base, &OsFilesystem),
        None
      );
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
        module("foo", None).resolve(&base, &OsFilesystem).unwrap(),
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
      module("foo", None).resolve(&base, &OsFilesystem).unwrap(),
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

    assert_eq!(module("foo", None).resolve(&base, &OsFilesystem), None);
  }

  #[test]
  fn implicit_justfile() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::create_dir(directory.path().join("foo")).unwrap();
    fs::write(directory.path().join("foo/justfile"), "").unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(
      module("foo", None).resolve(&base, &OsFilesystem).unwrap(),
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
      module("foo", None).resolve(&base, &OsFilesystem).unwrap(),
      directory.path().join("foo/mod.just"),
    );
  }

  #[test]
  fn implicit_no_file_returns_none() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(module("foo", None).resolve(&base, &OsFilesystem), None);
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
      module("foo", None).resolve(&base, &OsFilesystem).unwrap(),
      directory.path().join("foo.just"),
    );
  }
}
