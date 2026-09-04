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

  /// # Errors
  ///
  /// Returns an error if a shell-expanded path references an environment
  /// variable that cannot be read.
  pub fn resolve(&self, base_uri: &lsp::Url) -> Result<Option<PathBuf>> {
    let Some(StringLiteral {
      cooked,
      shell_expanded,
      ..
    }) = StringLiteral::parse(&self.path.value)
    else {
      return Ok(None);
    };

    let raw = if shell_expanded {
      shellexpand::full(&cooked)?.into_owned()
    } else {
      cooked
    };

    if raw.is_empty() {
      return Err(Error::EmptyImportPath);
    }

    Ok(if let Some(rest) = raw.strip_prefix("~/") {
      dirs::home_dir().map(|home| home.join(rest))
    } else {
      base_uri
        .to_file_path()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join(raw)))
    })
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
    let directory = Builder::new().prefix("just-lsp").tempdir().unwrap();

    let base =
      lsp::Url::from_file_path(directory.path().join("justfile")).unwrap();

    assert_eq!(
      import("'~/bar.just'").resolve(&base).unwrap().unwrap(),
      dirs::home_dir().unwrap().join("bar.just"),
    );
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
