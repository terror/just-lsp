use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ProjectDependencyTarget {
  Ambiguous(Vec<lsp::Url>),
  Cycle,
  Dynamic,
  Missing,
  Resolved(lsp::Url),
}
