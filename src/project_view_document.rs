use super::*;

#[derive(Debug)]
pub(super) struct ProjectViewDocument<'a> {
  pub(super) document: &'a Document,
  pub(super) load_depth: usize,
  pub(super) traversal_order: usize,
}
