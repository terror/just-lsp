use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectDependency {
  pub kind: DependencyKind,
  pub location: lsp::Range,
  pub target: DependencyTarget,
}
