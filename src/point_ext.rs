use super::*;

pub(crate) trait PointExt {
  fn advance(self, delta: Point) -> Self;
  fn position(&self) -> lsp::Position;
}

impl PointExt for Point {
  fn advance(self, delta: Point) -> Self {
    if delta.row == 0 {
      Point::new(self.row, self.column + delta.column)
    } else {
      Point::new(self.row + delta.row, delta.column)
    }
  }

  fn position(&self) -> lsp::Position {
    lsp::Position {
      line: u32::try_from(self.row).expect("line index exceeds u32::MAX"),
      character: u32::try_from(self.column)
        .expect("column index exceeds u32::MAX"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn advance_adds_columns_when_staying_on_same_row() {
    assert_eq!(Point::new(2, 3).advance(Point::new(0, 5)), Point::new(2, 8));
  }

  #[test]
  fn advance_moves_rows_and_resets_column_when_row_delta_positive() {
    assert_eq!(Point::new(1, 4).advance(Point::new(2, 3)), Point::new(3, 3));
  }
}
