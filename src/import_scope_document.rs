use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportScopeDocument {
  pub load_depth: usize,
  pub traversal_order: usize,
  pub uri: lsp::Url,
}
