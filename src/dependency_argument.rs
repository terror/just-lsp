use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyArgument {
  pub range: lsp::Range,
  pub starred: Option<lsp::Range>,
  pub value: String,
}
