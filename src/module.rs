#[derive(Debug, PartialEq)]
pub(crate) struct Module {
  pub(crate) name: String,
  pub(crate) optional: bool,
  pub(crate) path: Option<String>,
}
