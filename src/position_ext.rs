use super::*;

pub(crate) trait PositionExt {
  fn point(&self, document: &Document) -> Point;
}

impl PositionExt for lsp::Position {
  fn point(&self, document: &Document) -> Point {
    todo!()
  }
}
