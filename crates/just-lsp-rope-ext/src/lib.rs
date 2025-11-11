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

use {
  ropey::{self, Rope},
  std::convert::TryFrom,
  tower_lsp::lsp_types as lsp,
  tree_sitter::{InputEdit, Point},
};

#[derive(Clone, Debug, PartialEq)]
pub struct TextPosition {
  pub byte: usize,
  pub char: usize,
  pub code: usize,
  pub point: Point,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextEdit<'a> {
  pub end_char_idx: usize,
  pub input_edit: InputEdit,
  pub start_char_idx: usize,
  pub text: &'a str,
}

pub trait RopeExt {
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

    let range = change.range.unwrap_or_else(|| lsp::Range {
      start: self.byte_to_lsp_position(0),
      end: self.byte_to_lsp_position(text_end_byte_idx),
    });

    let start = self.lsp_position_to_core(range.start);
    let old_end = self.lsp_position_to_core(range.end);

    let new_end_byte = start.byte + text_end_byte_idx;

    let new_end_position = if new_end_byte >= self.len_bytes() {
      let line_idx = text.lines().count();

      let line_byte_idx = ropey::str_utils::line_to_byte_idx(text, line_idx);

      Point::new(
        self.len_lines() + line_idx,
        text_end_byte_idx - line_byte_idx,
      )
    } else {
      self.byte_to_tree_sitter_point(new_end_byte)
    };

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
    let row_idx = position.line as usize;
    let row_char_idx = self.line_to_char(row_idx);
    let row_byte_idx = self.line_to_byte(row_idx);
    let row_code_idx = self.char_to_utf16_cu(row_char_idx);

    let col_code_offset = position.character as usize;
    let col_code_idx = row_code_idx + col_code_offset;
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
}
