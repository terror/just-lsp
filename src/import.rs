#[derive(Debug, PartialEq)]
pub(crate) struct Import {
  pub(crate) optional: bool,
  pub(crate) path: String,
}
