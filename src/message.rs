use crate::common::*;

pub(crate) struct Message<'a> {
  pub(crate) content: &'a str,
  pub(crate) kind: lsp::MessageType,
}
