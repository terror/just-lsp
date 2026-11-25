use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Scope {
  Imported,
  Local,
  Module,
}

impl Scope {
  pub(crate) fn contains<T>(
    self,
    document: &lsp::Url,
    declaration: &Located<T>,
  ) -> bool {
    match self {
      Self::Imported => declaration.uri != *document,
      Self::Local => declaration.uri == *document,
      Self::Module => true,
    }
  }
}
