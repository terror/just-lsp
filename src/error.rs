use super::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("{0}")]
  Format(String),
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  LanguageError(#[from] tree_sitter::LanguageError),
}
