use super::*;

pub(crate) trait PointExt {
  fn advance(self, delta: Point) -> Self;
  fn position(&self, document: &Document) -> lsp::Position;
}

impl PointExt for Point {
  fn advance(self, delta: Point) -> Self {
    if delta.row == 0 {
      Point::new(self.row, self.column + delta.column)
    } else {
      Point::new(self.row + delta.row, delta.column)
    }
  }

  /// Tree-sitter points use a zero-based `row` plus UTF-8 byte offset
  /// `column`, while the LSP expects UTF-16 code-unit offsets.
  ///
  /// We take the document line for the point’s row, convert the byte column
  /// into a char index, and then into a UTF-16 offset to produce an `lsp::Position`.
  fn position(&self, document: &Document) -> lsp::Position {
    let line = document.content.line(self.row);

    let utf16_cu = line.char_to_utf16_cu(line.byte_to_char(self.column));

    lsp::Position {
      line: u32::try_from(self.row).expect("line index exceeds u32::MAX"),
      character: u32::try_from(utf16_cu)
        .expect("column index exceeds u32::MAX"),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  fn document(content: &str) -> Document {
    Document::try_from(lsp::DidOpenTextDocumentParams {
      text_document: lsp::TextDocumentItem {
        uri: lsp::Url::parse("file:///test.just").unwrap(),
        language_id: "just".to_string(),
        version: 1,
        text: content.to_string(),
      },
    })
    .unwrap()
  }

  #[test]
  fn advance_adds_columns_when_staying_on_same_row() {
    assert_eq!(Point::new(2, 3).advance(Point::new(0, 5)), Point::new(2, 8));
  }

  #[test]
  fn advance_moves_rows_and_resets_column_when_row_delta_positive() {
    assert_eq!(Point::new(1, 4).advance(Point::new(2, 3)), Point::new(3, 3));
  }

  #[test]
  fn converts_utf8_columns_to_utf16_offsets() {
    let document = document("a𐐀b");

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
