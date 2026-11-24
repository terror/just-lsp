use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ProjectDependencyTarget {
  Candidates(Vec<lsp::Url>),
  Dynamic,
  Missing,
  Path(lsp::Url),
}
