use super::*;

#[derive(Debug)]
pub(crate) struct MemoryFilesystem {
  files: BTreeMap<PathBuf, String>,
}

impl MemoryFilesystem {
  fn is_dir(&self, path: &Path) -> bool {
    self
      .files
      .keys()
      .any(|file| file != path && file.starts_with(path))
  }

  pub(crate) fn new(
    files: impl IntoIterator<Item = (PathBuf, String)>,
  ) -> Self {
    Self {
      files: files.into_iter().collect(),
    }
  }
}

impl Filesystem for MemoryFilesystem {
  fn is_file(&self, path: &Path) -> bool {
    self.files.contains_key(path)
  }

  fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
    if !self.is_dir(path) {
      return Err(io::ErrorKind::NotFound.into());
    }

    Ok(
      self
        .files
        .keys()
        .filter_map(|file| file.strip_prefix(path).ok()?.components().next())
        .map(|component| path.join(component))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect(),
    )
  }

  fn read_to_string(&self, path: &Path) -> io::Result<String> {
    self
      .files
      .get(path)
      .cloned()
      .ok_or_else(|| io::ErrorKind::NotFound.into())
  }
}
