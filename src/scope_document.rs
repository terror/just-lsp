#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScopeDocument<T> {
  pub(crate) document: T,
  pub(crate) load_depth: usize,
}
