use super::*;

pub struct Message<'a> {
  pub content: &'a str,
  pub kind: lsp::MessageType,
}
