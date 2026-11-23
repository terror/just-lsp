use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProjectDependency {
  pub(crate) kind: ProjectDependencyKind,
  pub(crate) location: lsp::Range,
  pub(crate) target: ProjectDependencyTarget,
}
