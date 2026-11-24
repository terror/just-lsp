use super::*;

pub(crate) trait Filesystem: Debug + Send + Sync {
  fn is_dir(&self, path: &Path) -> bool;

  fn is_file(&self, path: &Path) -> bool;

  fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;

  fn read_to_string(&self, path: &Path) -> io::Result<String>;
}
