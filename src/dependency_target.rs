use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum DependencyTarget {
  Ambiguous(Vec<lsp::Url>),
  Cycle,
  Dynamic,
  Missing,
  Resolved(lsp::Url),
}
