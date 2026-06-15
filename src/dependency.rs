use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Dependency {
  pub arguments: Vec<DependencyArgument>,
  pub mapped: Option<lsp::Range>,
  pub name: String,
  pub range: lsp::Range,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyArgument {
  pub range: lsp::Range,
  pub starred: Option<lsp::Range>,
  pub value: String,
}
