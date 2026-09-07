use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Located<T> {
  inner: T,
  pub uri: lsp::Url,
}

impl<T> Located<T> {
  #[must_use]
  pub fn as_ref(&self) -> Located<&T> {
    Located::new(self.uri.clone(), &self.inner)
  }

  #[must_use]
  pub fn into_inner(self) -> T {
    self.inner
  }

  #[must_use]
  pub fn location(&self, range: lsp::Range) -> lsp::Location {
    lsp::Location::new(self.uri.clone(), range)
  }

  #[must_use]
  pub fn new(uri: lsp::Url, inner: T) -> Self {
    Self { inner, uri }
  }
}

impl<T> Deref for Located<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}
