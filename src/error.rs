use super::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("{0}")]
  Format(String),
  #[error("document URI `{0}` is not a file URI")]
  InvalidDocumentUri(lsp::Url),
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  LanguageError(#[from] tree_sitter::LanguageError),
}
