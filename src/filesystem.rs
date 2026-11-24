use super::*;

#[cfg(test)]
pub(crate) use memory_filesystem::MemoryFilesystem;
pub(crate) use os_filesystem::OsFilesystem;

#[cfg(test)]
mod memory_filesystem;
mod os_filesystem;

pub(crate) trait Filesystem: Debug + Send + Sync {
  fn is_file(&self, path: &Path) -> bool;

  fn justfiles(&self, directory: &Path) -> Vec<PathBuf> {
    let mut justfiles = self
      .read_dir(directory)
      .unwrap_or_default()
      .into_iter()
      .filter(|path| path.is_justfile() && self.is_file(path))
      .collect::<Vec<_>>();

    justfiles.sort();

    justfiles
  }

  fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;

  fn read_to_string(&self, path: &Path) -> io::Result<String>;
}
