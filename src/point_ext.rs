use super::*;

pub(crate) trait PointExt {
  fn position(&self) -> lsp::Position;
}

impl PointExt for Point {
  fn position(&self) -> lsp::Position {
    lsp::Position {
      line: u32::try_from(self.row).expect("line index exceeds u32::MAX"),
      character: u32::try_from(self.column)
        .expect("column index exceeds u32::MAX"),
    }
  }
}
