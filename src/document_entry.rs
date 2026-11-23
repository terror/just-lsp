use super::*;

#[derive(Debug)]
pub(super) struct DocumentEntry {
  pub(super) document: Document,
  pub(super) imports: OnceLock<Vec<ProjectDependency>>,
  pub(super) modules: OnceLock<Vec<ModuleDeclaration>>,
  pub(super) open: bool,
}

impl DocumentEntry {
  pub(super) fn new(document: Document, open: bool) -> Self {
    Self {
      document,
      imports: OnceLock::new(),
      modules: OnceLock::new(),
      open,
    }
  }
}
