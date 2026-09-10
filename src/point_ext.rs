use super::*;

pub trait PointExt {
  #[must_use]
  fn advance(self, delta: Point) -> Self;
  fn position(&self, document: &Document) -> lsp::Position;
}

impl PointExt for Point {
  /// Returns a new point shifted by `delta`, resetting the column when the
  /// delta moves to a different row.
  fn advance(self, delta: Point) -> Self {
    if delta.row == 0 {
      Point::new(self.row, self.column + delta.column)
    } else {
      Point::new(self.row + delta.row, delta.column)
    }
  }

  /// Tree-sitter points use a zero-based `row` plus UTF-8 byte offset `column`,
  /// while the LSP expects UTF-16 code-unit offsets.
  ///
  /// We take the document line for the point’s row, convert the byte column
  /// into a char index, and then into a UTF-16 offset to produce an
  /// `lsp::Position`.
  fn position(&self, document: &Document) -> lsp::Position {
    let byte = if self.row == 0 {
      0
    } else {
      document
        .content
        .bytes()
        .enumerate()
        .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1))
        .nth(self.row - 1)
        .expect("line index out of bounds")
    };

    document.content.byte_to_lsp_position(byte + self.column)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn advance_adds_columns_when_staying_on_same_row() {
    assert_eq!(Point::new(2, 3).advance(Point::new(0, 5)), Point::new(2, 8));
  }

  #[test]
  fn advance_moves_rows_and_resets_column_when_row_delta_positive() {
    assert_eq!(Point::new(1, 4).advance(Point::new(2, 3)), Point::new(3, 3));
  }

  #[test]
  fn bare_carriage_returns_preserve_lsp_lines() {
    let document = Document::from("foo\r🧪\r\nbar\rbaz\nqux\r");

    for (point, position) in [
      (Point::new(0, 8), lsp::Position::new(1, 2)),
      (Point::new(1, 4), lsp::Position::new(3, 0)),
      (Point::new(2, 4), lsp::Position::new(5, 0)),
    ] {
      assert_eq!(point.position(&document), position);
    }
  }

  #[test]
  fn converts_utf8_columns_to_utf16_offsets() {
    let document = Document::from("a𐐀b");

    assert_eq!(
      Point::new(0, document.content.line(0).char_to_byte(2))
        .position(&document),
      lsp::Position {
        line: 0,
        character: 3
      }
    );
  }
}
