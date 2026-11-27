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
    self.path.value.starts_with('f')
  }

  /// Returns the filesystem path of the imported file.
  ///
  /// The path literal is unquoted and its escape sequences are processed.
  /// Shell-expanded literals, such as `x'$FOO/bar.just'`, also have
  /// environment variables and a leading `~` expanded. A path beginning with
  /// `~/` is resolved against the home directory, and any other relative path
  /// is resolved against the directory containing `base_uri`. Absolute paths
  /// are returned as is. The file is not required to exist.
  ///
  /// Returns `None` if the path is not a complete string literal, if it
  /// begins with `~/` and the home directory is unknown, or if `base_uri` is
  /// not a `file` URL.
  ///
  /// # Errors
  ///
  /// Returns an error if the path is empty, contains an invalid escape
  /// sequence, or is shell-expanded and references an environment variable
  /// that cannot be read.
  ///
  /// # Example
  ///
  /// ```
  /// use {
  ///   just_lsp::Document,
  ///   tower_lsp::lsp_types::Url,
  /// };
  ///
  /// let directory = std::env::temp_dir();
  ///
  /// let base = Url::from_file_path(directory.join("justfile")).unwrap();
  ///
  /// let document = Document::from("import 'foo.just'\n");
  ///
  /// assert_eq!(
  ///   document.imports()[0].resolve(&base).unwrap(),
  ///   Some(directory.join("foo.just")),
  /// );
  /// ```
  pub fn resolve(&self, base_uri: &lsp::Url) -> Result<Option<PathBuf>> {
    StringLiteral::resolve_path(&self.path.value, base_uri)
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
  fn empty_literal_returns_error() {
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert!(matches!(
      import("''").resolve(&base),
      Err(Error::EmptyImportPath),
    ));

    assert!(matches!(
      import("\"\"").resolve(&base),
      Err(Error::EmptyImportPath),
    ));

    assert_eq!(import("").resolve(&base).unwrap(), None);
  }

  #[test]
  fn home_directory() {
    #[track_caller]
    fn case(source: &str, expected: &str) {
      let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

      let base =
        lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

      assert_eq!(
        import(source).resolve(&base).unwrap().unwrap(),
        env::home_dir().unwrap().join(expected),
      );
    }

    case("'~/bar.just'", "bar.just");
    case("x'~/bar.just'", "bar.just");
    case("x'~'", "");
  }

  #[test]
  fn resolve() {
    #[track_caller]
    fn case(source: &str, expected: &str) {
      let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

      let base =
        lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

      assert_eq!(
        import(source).resolve(&base).unwrap(),
        Some(directory.path().join(expected)),
      );
    }

    case("'foo.just'", "foo.just");
    case("\"foo.just\"", "foo.just");
    case("'foo/bar.just'", "foo/bar.just");
    case("x'foo.just'", "foo.just");
    case("x'''foo.just'''", "foo.just");
    case("x\"\"\"foo.just\"\"\"", "foo.just");
  }
}
