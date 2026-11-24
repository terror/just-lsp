use super::*;

#[derive(Debug)]
pub(crate) struct OsFilesystem;

impl Filesystem for OsFilesystem {
  fn is_file(&self, path: &Path) -> bool {
    path.is_file()
  }

  fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
    Ok(
      fs::read_dir(path)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect(),
    )
  }

  fn read_to_string(&self, path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
  }
}
