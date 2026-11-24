use super::*;

#[cfg(test)]
pub(crate) use memory_filesystem::MemoryFilesystem;
pub(crate) use os_filesystem::OsFilesystem;

#[cfg(test)]
mod memory_filesystem;
mod os_filesystem;

pub(crate) trait Filesystem: Debug + Send + Sync {
  fn is_file(&self, path: &Path) -> bool;

  fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;

  fn read_to_string(&self, path: &Path) -> io::Result<String>;
}
