use super::*;

pub(crate) trait UrlExt: Sized {
  fn file_path(&self) -> Result<PathBuf>;
  fn from_path(path: &Path) -> Option<Self>;
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
}
