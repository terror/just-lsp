use super::*;

pub(crate) trait NodeExt {
  fn get_range(&self) -> lsp::Range;
}

impl NodeExt for Node<'_> {
  fn get_range(&self) -> lsp::Range {
    lsp::Range {
      start: self.start_position().position(),
      end: self.end_position().position(),
    }
  }
}
