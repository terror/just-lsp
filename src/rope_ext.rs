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

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Position {
  /// Absolute UTF-8 byte offset into the rope (start-of-doc origin).
  pub(crate) byte: usize,
  /// Absolute Unicode scalar index used by `ropey` for slicing.
  pub(crate) char: usize,
  /// Absolute UTF-16 code-unit offset for LSP column conversions.
  pub(crate) code: usize,
  /// Tree-sitter line/column (column measured in bytes from line start).
  pub(crate) point: Point,
}

impl From<&Rope> for Position {
  fn from(value: &Rope) -> Self {
    let (end_byte, end_char) = (value.len_bytes(), value.len_chars());

    Self {
      byte: end_byte,
      char: end_char,
      code: value.char_to_utf16_cu(end_char),
      point: value.byte_to_tree_sitter_point(end_byte),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Edit<'a> {
  /// Rope character index where the edit starts.
  pub(crate) end_char_idx: usize,
  /// Tree-sitter edit metadata (byte + point bookkeeping).
  pub(crate) input_edit: InputEdit,
  /// Rope character index where the edit ends (exclusive).
  pub(crate) start_char_idx: usize,
  /// Replacement text from the LSP change event.
  pub(crate) text: &'a str,
}

pub(crate) trait RopeExt {
  /// Applies a previously constructed [`Edit`] to the rope, keeping both
  /// the textual contents and the internal tree-sitter offsets in sync.
  fn apply_edit(&mut self, edit: &Edit);

  /// Converts an LSP `textDocument/didChange` event into a [`Edit`] that
  /// can be consumed both by `ropey` and tree-sitter.
  fn build_edit<'a>(
    &self,
    change: &'a lsp::TextDocumentContentChangeEvent,
  ) -> Edit<'a>;

  /// Maps an absolute byte offset into an LSP line/character pair where the
  /// column is expressed in UTF-16 code units as required by the spec.
  fn byte_to_lsp_position(&self, offset: usize) -> lsp::Position;

  /// Maps an absolute byte offset into a tree-sitter [`Point`] (line and utf8
  /// column measured in bytes).
  fn byte_to_tree_sitter_point(&self, offset: usize) -> Point;

  /// Converts an LSP position back into absolute byte/char/code offsets and a
  /// tree-sitter point so downstream consumers can choose whichever coordinate
  /// space they need.
  fn lsp_position_to_position(&self, position: lsp::Position) -> Position;

  /// Converts an LSP `Range` (UTF-16 line/column pairs) into our richer
  /// [`Position`] objects so we know the corresponding byte/char/code/point
  /// offsets for both the start and end of the change span.
  fn lsp_range_to_range(&self, range: lsp::Range) -> (Position, Position) {
    (
      self.lsp_position_to_position(range.start),
      self.lsp_position_to_position(range.end),
    )
  }
}

impl RopeExt for Rope {
  fn apply_edit(&mut self, edit: &Edit) {
    self.remove(edit.start_char_idx..edit.end_char_idx);

    if !edit.text.is_empty() {
      self.insert(edit.start_char_idx, edit.text);
    }
  }

  fn build_edit<'a>(
    &self,
    change: &'a lsp::TextDocumentContentChangeEvent,
  ) -> Edit<'a> {
    let text = change.text.as_str();

    let (start, old_end) = match change.range {
      Some(range) => self.lsp_range_to_range(range),
      None => (Position::default(), Position::from(self)),
    };

    let input_edit = InputEdit {
      new_end_byte: start.byte + text.len(),
      new_end_position: start.point.advance(text.point_delta()),
      old_end_byte: old_end.byte,
      old_end_position: old_end.point,
      start_byte: start.byte,
      start_position: start.point,
    };

    Edit {
      end_char_idx: old_end.char,
      input_edit,
      start_char_idx: start.char,
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
    Point::new(line_idx, byte_idx - self.line_to_byte(line_idx))
  }

  fn lsp_position_to_position(&self, position: lsp::Position) -> Position {
    let line_count = self.len_lines();

    let row_idx = if line_count == 0 {
      0
    } else {
      cmp::min(position.line as usize, line_count - 1)
    };

    let row_char_idx = self.line_to_char(row_idx);
    let row_byte_idx = self.line_to_byte(row_idx);
    let row_code_idx = self.char_to_utf16_cu(row_char_idx);

    let col_code_offset = cmp::min(
      position.character as usize,
      self.line(row_idx).len_utf16_cu(),
    );

    let col_code_idx = row_code_idx + col_code_offset;
    let col_char_idx = self.utf16_cu_to_char(col_code_idx);
    let col_byte_idx = self.char_to_byte(col_char_idx);

    Position {
      char: col_char_idx,
      byte: col_byte_idx,
      code: col_code_idx,
      point: Point::new(row_idx, col_byte_idx - row_byte_idx),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, ropey::Rope};

  type Range = (u32, u32, u32, u32);

  fn change_event(
    text: &str,
    (start_line, start_character, end_line, end_character): Range,
  ) -> lsp::TextDocumentContentChangeEvent {
    lsp::TextDocumentContentChangeEvent {
      range: Some(lsp::Range {
        start: lsp::Position::new(start_line, start_character),
        end: lsp::Position::new(end_line, end_character),
      }),
      range_length: None,
      text: text.into(),
    }
  }

  #[test]
  fn apply_edit_updates_rope_contents() {
    let mut rope = Rope::from_str("hello world");

    rope.apply_edit(&rope.build_edit(&change_event("rope", (0, 6, 0, 11))));

    assert_eq!(rope.to_string(), "hello rope");
  }

  #[test]
  fn round_trip_handles_utf16_columns() {
    let rope = Rope::from_str("a😊b\nsecond");

    let after_emoji = rope.to_string().find('b').unwrap();

    let position = rope.byte_to_lsp_position(after_emoji);

    let core = rope.lsp_position_to_position(position);

    assert_eq!(core.byte, after_emoji);
    assert_eq!(core.char, rope.byte_to_char(after_emoji));
    assert_eq!(core.code, rope.char_to_utf16_cu(core.char));
    assert_eq!(core.point, rope.byte_to_tree_sitter_point(after_emoji));
  }

  #[test]
  fn build_edit_populates_input_edit_fields() {
    let rope = Rope::from_str("hello\nworld\n");

    let change = change_event("rust", (1, 0, 1, 5));

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

    let change = change_event(insertion, (0, 5, 0, 5));

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

    let change = change_event(insertion, (0, 5, 0, 5));

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

    let core = rope.lsp_position_to_position(line_past_end);

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

    let core = rope.lsp_position_to_position(lsp::Position::new(
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
