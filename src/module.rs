use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Module {
  pub(crate) name: String,
  pub(crate) name_range: lsp::Range,
  pub(crate) optional: bool,
  pub(crate) path: Option<String>,
}
