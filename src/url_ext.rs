use super::*;

pub trait UrlExt: Sized {
  /// # Errors
  ///
  /// Returns an [`Error`] if the URL is not a valid file path.
  fn file_path(&self) -> Result<PathBuf>;

  fn from_path(path: &Path) -> Option<Self>;

  #[must_use]
  fn normalize(&self) -> Self;
}

impl UrlExt for lsp::Url {
  fn file_path(&self) -> Result<PathBuf> {
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
      self
        .to_file_path()
        .map_err(|()| Error::InvalidDocumentUri(self.clone()))
    }

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
      Err(Error::Io(std::io::ErrorKind::Unsupported.into()))
    }
  }

  fn from_path(path: &Path) -> Option<Self> {
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
      Self::from_file_path(path).ok()
    }

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
      let _ = path;
      None
    }
  }

  fn normalize(&self) -> Self {
    self
      .file_path()
      .ok()
      .and_then(|path| Self::from_path(&path))
      .unwrap_or_else(|| self.clone())
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn normalize() {
    #[track_caller]
    fn case(uri: &str, expected: &str) {
      assert_eq!(lsp::Url::parse(uri).unwrap().normalize().as_str(), expected);
    }

    if cfg!(windows) {
      case("file:///c%3A/foo%40bar", "file:///C:/foo@bar");
      case("file:///c:/foo", "file:///C:/foo");
    } else {
      case("file:///foo%40bar", "file:///foo@bar");
      case("file:///foo%c3%a9", "file:///foo%C3%A9");
    }

    case("untitled:foo%40bar", "untitled:foo%40bar");
  }
}
