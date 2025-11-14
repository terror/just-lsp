use super::*;

pub(crate) trait PositionExt {
  fn point(&self, document: &Document) -> Point;
}

impl PositionExt for lsp::Position {
  /// LSP positions use a zero-based line index for `line` and a UTF-16
  /// code-unit offset within that line for `character`.
  ///
  /// Ropey and Tree-sitter, however, operate on UTF-8 byte offsets. To bridge
  /// this mismatch, we take the line number directly as the Tree-sitter `row`,
  /// then look up the corresponding line in the Rope and convert the UTF-16
  /// `character` offset into a char index and, from there, into a UTF-8 byte
  /// offset for the `column`.
  ///
  /// The resulting `(row, column)` byte position is then used to locate the
  /// node in the syntax tree.
  fn point(&self, document: &Document) -> Point {
    let row =
      usize::try_from(self.line).expect("line index exceeds usize::MAX");

    let line = document.content.line(row);

    let utf16_cu = usize::try_from(self.character)
      .expect("character index exceeds usize::MAX");

    Point {
      row,
      column: line.char_to_byte(line.utf16_cu_to_char(utf16_cu)),
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
  fn converts_utf16_offsets_to_utf8_columns() {
    let doc = document("a🧪b");

    let position = lsp::Position {
      line: 0,
      character: 3,
    };

    let point = position.point(&doc);

    assert_eq!(point, Point { row: 0, column: 5 });
  }
}
