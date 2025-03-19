use super::*;

pub(crate) trait PointExt {
  fn position(&self) -> lsp::Position;
}

impl PointExt for Point {
  fn position(&self) -> lsp::Position {
    lsp::Position {
      line: self.row as u32,
      character: self.column as u32,
    }
  }
}
