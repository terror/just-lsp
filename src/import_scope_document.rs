use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImportScopeDocument {
  pub(crate) load_depth: usize,
  pub(crate) uri: lsp::Url,
}
