use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Located<T> {
  pub uri: lsp::Url,
  pub value: T,
}
