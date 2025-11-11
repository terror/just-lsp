use super::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Language(#[from] LanguageError),
}
