use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ProjectDependencyTarget {
  Cycle,
  Dynamic,
  Missing,
  Resolved(lsp::Url),
}
