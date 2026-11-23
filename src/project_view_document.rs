use super::*;

#[derive(Clone, Debug)]
pub(super) struct ProjectViewDocument<'a> {
  pub(super) document: &'a Document,
  pub(super) load_depth: usize,
}
