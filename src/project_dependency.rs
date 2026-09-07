use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectDependency {
  pub kind: ProjectDependencyKind,
  pub location: lsp::Range,
  pub target: ProjectDependencyTarget,
}
