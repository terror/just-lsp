use super::*;

pub trait PositionExt {
  fn point(&self) -> Point;
}

impl PositionExt for lsp::Position {
  fn point(&self) -> Point {
    Point {
      row: self.line as usize,
      column: self.character as usize,
    }
  }
}
