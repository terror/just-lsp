use super::*;

pub(crate) trait PointExt {
  fn position(&self, document: &Document) -> lsp::Position;
}

impl PointExt for Point {
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
  fn converts_utf8_columns_to_utf16_offsets() {
    let document = document("a𐐀b");

    assert_eq!(
      Point::new(0, document.content.line(0).char_to_byte(2)).position(&document),
      lsp::Position {
        line: 0,
        character: 3
      }
    );
  }
}
