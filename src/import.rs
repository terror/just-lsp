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

  pub(crate) fn is_enabled(&self) -> bool {
    self
      .attributes
      .iter()
      .filter_map(Attribute::condition)
      .reduce(|left, right| left || right)
      .unwrap_or(true)
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
    }) = StringLiteral::parse(&self.path.value)?
    else {
      return Ok(None);
    };

    let raw = if shell_expanded {
      shellexpand::full_with_context(
        &cooked,
        || env::home_dir()?.into_os_string().into_string().ok(),
        |name| env::var(name).map(Some),
      )?
      .into_owned()
    } else {
      cooked
    };

    if raw.is_empty() {
      return Err(Error::EmptyImportPath);
    }

    Ok(if let Some(rest) = raw.strip_prefix("~/") {
      env::home_dir().map(|home| home.join(rest))
    } else {
      base_uri
        .file_path()
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
