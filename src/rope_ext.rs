//! Extensions that bridge `ropey::Rope` with Language Server Protocol positions
//! and tree-sitter edit bookkeeping.
//!
//! The [`RopeExt`] trait is used inside `just-lsp` to keep three different
//! coordinate spaces (bytes, UTF-16 code units, and tree-sitter points) in sync
//! whenever an editor sends a `textDocument/didChange` notification.
//!
//! ```
//! use {
//!   just_lsp_rope_ext::RopeExt,
//!   ropey::Rope,
//!   tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent}
//! };
//!
//! let mut rope = Rope::from_str("hello world");
//!
//! let change = TextDocumentContentChangeEvent {
//!   range: Some(Range {
//!     start: Position::new(0, 6),
//!     end: Position::new(0, 11),
//!   }),
//!   range_length: None,
//!   text: "rope".into(),
//! };
//!
//! let edit = rope.build_edit(&change);
//! rope.apply_edit(&edit);
//!
//! assert_eq!(rope.to_string(), "hello rope");
//! ```

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextPosition {
  pub(crate) byte: usize,
  pub(crate) char: usize,
  pub(crate) code: usize,
  pub(crate) point: Point,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextEdit<'a> {
  pub(crate) end_char_idx: usize,
  pub(crate) input_edit: InputEdit,
  pub(crate) start_char_idx: usize,
  pub(crate) text: &'a str,
}

pub(crate) trait RopeExt {
  /// Applies a previously constructed [`TextEdit`] to the rope, keeping both
  /// the textual contents and the internal tree-sitter offsets in sync.
  fn apply_edit(&mut self, edit: &TextEdit);

  /// Converts an LSP `textDocument/didChange` event into a [`TextEdit`] that
  /// can be consumed both by `ropey` and tree-sitter.
  fn build_edit<'a>(
    &self,
    change: &'a lsp::TextDocumentContentChangeEvent,
  ) -> TextEdit<'a>;

  /// Maps an absolute byte offset into an LSP line/character pair where the
  /// column is expressed in UTF-16 code units as required by the spec.
  fn byte_to_lsp_position(&self, offset: usize) -> lsp::Position;

  /// Maps an absolute byte offset into a tree-sitter [`Point`] (line and utf8
  /// column measured in bytes).
  fn byte_to_tree_sitter_point(&self, offset: usize) -> Point;

  /// Converts an LSP position back into absolute byte/char/code offsets and a
  /// tree-sitter point so downstream consumers can choose whichever coordinate
  /// space they need.
  fn lsp_position_to_core(&self, position: lsp::Position) -> TextPosition;
}

impl RopeExt for Rope {
  fn apply_edit(&mut self, edit: &TextEdit) {
    self.remove(edit.start_char_idx..edit.end_char_idx);

    if !edit.text.is_empty() {
      self.insert(edit.start_char_idx, edit.text);
    }
  }

  fn build_edit<'a>(
    &self,
    change: &'a lsp::TextDocumentContentChangeEvent,
  ) -> TextEdit<'a> {
    let text = change.text.as_str();
    let text_end_byte_idx = text.len();

    let (start, old_end) = if let Some(range) = change.range {
      (
        self.lsp_position_to_core(range.start),
        self.lsp_position_to_core(range.end),
      )
    } else {
      let (doc_end_char, doc_end_byte) = (self.len_chars(), self.len_bytes());

      let start = TextPosition {
        byte: 0,
        char: 0,
        code: 0,
        point: Point::new(0, 0),
      };

      let old_end = TextPosition {
        char: doc_end_char,
        byte: doc_end_byte,
        code: self.char_to_utf16_cu(doc_end_char),
        point: self.byte_to_tree_sitter_point(doc_end_byte),
      };

      (start, old_end)
    };

    let new_end_byte = start.byte + text_end_byte_idx;
    let new_end_position = point_after_insertion(start.point, text);

    let input_edit = InputEdit {
      start_byte: start.byte,
      old_end_byte: old_end.byte,
      new_end_byte,
      start_position: start.point,
      old_end_position: old_end.point,
      new_end_position,
    };

    TextEdit {
      input_edit,
      start_char_idx: start.char,
      end_char_idx: old_end.char,
      text,
    }
  }

  fn byte_to_lsp_position(&self, byte_idx: usize) -> lsp::Position {
    let line_idx = self.byte_to_line(byte_idx);

    let line_char_idx = self.line_to_char(line_idx);
    let line_utf16_cu_idx = self.char_to_utf16_cu(line_char_idx);

    let char_idx = self.byte_to_char(byte_idx);
    let char_utf16_cu_idx = self.char_to_utf16_cu(char_idx);

    let character = char_utf16_cu_idx - line_utf16_cu_idx;

    lsp::Position::new(
      u32::try_from(line_idx).expect("line index exceeds u32::MAX"),
      u32::try_from(character).expect("character offset exceeds u32::MAX"),
    )
  }

  fn byte_to_tree_sitter_point(&self, byte_idx: usize) -> Point {
    let line_idx = self.byte_to_line(byte_idx);
    let line_byte_idx = self.line_to_byte(line_idx);
    Point::new(line_idx, byte_idx - line_byte_idx)
  }

  fn lsp_position_to_core(&self, position: lsp::Position) -> TextPosition {
    let requested_row = position.line as usize;

    let line_count = self.len_lines();
    let row_idx = requested_row.min(line_count.saturating_sub(1));

    let row_char_idx = self.line_to_char(row_idx);
    let row_byte_idx = self.line_to_byte(row_idx);
    let row_code_idx = self.char_to_utf16_cu(row_char_idx);

    let row_end_char_idx = if row_idx + 1 < line_count {
      self.line_to_char(row_idx + 1)
    } else {
      self.len_chars()
    };

    let row_end_code_idx = self.char_to_utf16_cu(row_end_char_idx);

    let col_code_offset = position.character as usize;
    let unclamped_code_idx = row_code_idx + col_code_offset;
    let col_code_idx = unclamped_code_idx.min(row_end_code_idx);

    let col_char_idx = self.utf16_cu_to_char(col_code_idx);
    let col_byte_idx = self.char_to_byte(col_char_idx);

    TextPosition {
      char: col_char_idx,
      byte: col_byte_idx,
      code: col_code_idx,
      point: tree_sitter::Point::new(row_idx, col_byte_idx - row_byte_idx),
    }
  }
}

fn point_after_insertion(start: Point, text: &str) -> Point {
  let (mut row, mut column) = (start.row, start.column);

  let mut chars = text.chars().peekable();

  while let Some(ch) = chars.next() {
    match ch {
      '\r' => {
        if matches!(chars.peek().copied(), Some('\n')) {
          chars.next();
        }

        row += 1;
        column = 0;
      }
      '\n' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}' => {
        row += 1;
        column = 0;
      }
      _ => {
        column += ch.len_utf8();
      }
    }
  }

  Point::new(row, column)
}

#[cfg(test)]
mod tests {
  use {super::*, ropey::Rope};

  fn change_event(
    range: lsp::Range,
    text: &str,
  ) -> lsp::TextDocumentContentChangeEvent {
    lsp::TextDocumentContentChangeEvent {
      range: Some(range),
      range_length: None,
      text: text.into(),
    }
  }

  #[test]
  fn apply_edit_updates_rope_contents() {
    let mut rope = Rope::from_str("hello world");

    let change = change_event(
      lsp::Range {
        start: lsp::Position::new(0, 6),
        end: lsp::Position::new(0, 11),
      },
      "rope",
    );

    rope.apply_edit(&rope.build_edit(&change));

    assert_eq!(rope.to_string(), "hello rope");
  }

  #[test]
  fn lsp_round_trip_handles_utf16_columns() {
    let rope = Rope::from_str("a😊b\nsecond");

    let after_emoji = rope.to_string().find('b').unwrap();

    let position = rope.byte_to_lsp_position(after_emoji);

    let core = rope.lsp_position_to_core(position);

    assert_eq!(core.byte, after_emoji);
    assert_eq!(core.char, rope.byte_to_char(after_emoji));
    assert_eq!(core.code, rope.char_to_utf16_cu(core.char));
    assert_eq!(core.point, rope.byte_to_tree_sitter_point(after_emoji));
  }

  #[test]
  fn build_edit_populates_input_edit_fields() {
    let rope = Rope::from_str("hello\nworld\n");

    let change = change_event(
      lsp::Range {
        start: lsp::Position::new(1, 0),
        end: lsp::Position::new(1, 5),
      },
      "rust",
    );

    let edit = rope.build_edit(&change);

    let expected_start_byte = rope.line_to_byte(1);
    let expected_start_char = rope.line_to_char(1);

    assert_eq!(edit.start_char_idx, expected_start_char);
    assert_eq!(edit.end_char_idx, expected_start_char + 5);
    assert_eq!(edit.input_edit.start_byte, expected_start_byte);
    assert_eq!(edit.input_edit.old_end_byte, expected_start_byte + 5);
    assert_eq!(edit.input_edit.new_end_byte, expected_start_byte + 4);
    assert_eq!(edit.input_edit.start_position, Point::new(1, 0));
    assert_eq!(edit.input_edit.old_end_position, Point::new(1, 5));
    assert_eq!(edit.input_edit.new_end_position, Point::new(1, 4));
  }

  #[test]
  fn build_edit_handles_whole_document_replace() {
    let rope = Rope::from_str("hello\nworld\n");
    let replacement = "only\nnew";

    let change = lsp::TextDocumentContentChangeEvent {
      range: None,
      range_length: None,
      text: replacement.into(),
    };

    let edit = rope.build_edit(&change);

    assert_eq!(edit.start_char_idx, 0);
    assert_eq!(edit.end_char_idx, rope.len_chars());

    assert_eq!(edit.input_edit.start_byte, 0);
    assert_eq!(edit.input_edit.old_end_byte, rope.len_bytes());
    assert_eq!(edit.input_edit.new_end_byte, replacement.len());
    assert_eq!(edit.input_edit.start_position, Point::new(0, 0));

    assert_eq!(
      edit.input_edit.old_end_position,
      rope.byte_to_tree_sitter_point(rope.len_bytes()),
    );

    assert_eq!(edit.input_edit.new_end_position, Point::new(1, 3));
    assert_eq!(edit.text, replacement);
  }

  #[test]
  fn build_edit_computes_new_point_from_inserted_text() {
    let rope = Rope::from_str("hello\nworld\n");

    let insertion = "\nbrave\nnew";

    let change = change_event(
      lsp::Range {
        start: lsp::Position::new(0, 5),
        end: lsp::Position::new(0, 5),
      },
      insertion,
    );

    let edit = rope.build_edit(&change);

    let line_start_byte = rope.line_to_byte(0);

    assert_eq!(edit.input_edit.start_byte, line_start_byte + 5);

    assert_eq!(
      edit.input_edit.new_end_byte,
      line_start_byte + 5 + insertion.len(),
    );

    assert_eq!(edit.input_edit.start_position, Point::new(0, 5));
    assert_eq!(edit.input_edit.new_end_position, Point::new(2, 3));
    assert_eq!(edit.input_edit.old_end_position, Point::new(0, 5));
  }

  #[test]
  fn build_edit_handles_crlf_and_multibyte_insertions() {
    let rope = Rope::from_str("hello\nworld\n");

    let insertion = "\r\n😊é";

    let change = change_event(
      lsp::Range {
        start: lsp::Position::new(0, 5),
        end: lsp::Position::new(0, 5),
      },
      insertion,
    );

    let edit = rope.build_edit(&change);

    let start_byte = rope.line_to_byte(0) + 5;
    let start_char = rope.line_to_char(0) + 5;
    let multibyte_tail = "😊é";

    assert_eq!(edit.start_char_idx, start_char);
    assert_eq!(edit.end_char_idx, start_char);
    assert_eq!(edit.input_edit.start_byte, start_byte);
    assert_eq!(edit.input_edit.new_end_byte, start_byte + insertion.len());
    assert_eq!(edit.input_edit.start_position, Point::new(0, 5));

    assert_eq!(
      edit.input_edit.new_end_position,
      Point::new(1, multibyte_tail.len()),
    );
  }

  #[test]
  fn lsp_position_to_core_clamps_line_index() {
    let rope = Rope::from_str("hello\nworld");

    let line_past_end = lsp::Position::new(42, 0);

    let core = rope.lsp_position_to_core(line_past_end);

    let last_line_idx = rope.len_lines() - 1;
    let last_line_char = rope.line_to_char(last_line_idx);
    let last_line_byte = rope.line_to_byte(last_line_idx);
    let last_line_code = rope.char_to_utf16_cu(last_line_char);

    assert_eq!(core.point, Point::new(last_line_idx, 0));
    assert_eq!(core.byte, last_line_byte);
    assert_eq!(core.char, last_line_char);
    assert_eq!(core.code, last_line_code);
  }

  #[test]
  fn lsp_position_to_core_clamps_column_index() {
    let rope = Rope::from_str("a😊b\nsecond");

    let line_idx = 0;

    let core = rope.lsp_position_to_core(lsp::Position::new(
      u32::try_from(line_idx).unwrap(),
      100,
    ));

    let line_start_char = rope.line_to_char(line_idx);
    let line_start_byte = rope.char_to_byte(line_start_char);

    let line_end_char = rope.line_to_char(line_idx + 1);
    let line_end_byte = rope.char_to_byte(line_end_char);
    let line_end_code = rope.char_to_utf16_cu(line_end_char);

    assert_eq!(
      core.point,
      Point::new(line_idx, line_end_byte - line_start_byte)
    );

    assert_eq!(core.byte, line_end_byte);
    assert_eq!(core.char, line_end_char);
    assert_eq!(core.code, line_end_code);
  }
}
