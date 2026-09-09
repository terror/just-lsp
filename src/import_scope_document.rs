use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportScopeDocument {
  pub load_depth: usize,
  pub uri: lsp::Url,
}
