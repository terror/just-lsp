use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Dependency {
  pub arguments: Vec<DependencyArgument>,
  pub mapped: Option<lsp::Range>,
  pub name: String,
  pub phase: DependencyPhase,
  pub range: lsp::Range,
}
