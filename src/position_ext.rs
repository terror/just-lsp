use super::*;

pub(crate) trait PositionExt {
  fn point(&self, document: &Document) -> Point;
}

impl PositionExt for lsp::Position {
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
