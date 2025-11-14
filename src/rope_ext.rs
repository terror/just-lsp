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
pub(crate) struct Position {
  pub(crate) byte: usize,
  pub(crate) char: usize,
  pub(crate) point: Point,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Edit<'a> {
  pub(crate) end_char: usize,
  pub(crate) input_edit: InputEdit,
  pub(crate) start_char: usize,
  pub(crate) text: &'a str,
}

pub(crate) trait RopeExt {
  fn apply_edit(&mut self, edit: &Edit);
  fn build_edit<'a>(
    &self,
    change: &'a lsp::TextDocumentContentChangeEvent,
  ) -> Edit<'a>;
  fn byte_to_lsp_position(&self, offset: usize) -> lsp::Position;
  fn lsp_position_to_position(&self, position: lsp::Position) -> Position;
}

impl RopeExt for Rope {
  /// Applies a previously constructed [`Edit`] to the rope, keeping both
  /// the textual contents and the internal tree-sitter offsets in sync.
  fn apply_edit(&mut self, edit: &Edit) {
    self.remove(edit.start_char..edit.end_char);

    if !edit.text.is_empty() {
      self.insert(edit.start_char, edit.text);
    }
  }

  /// Converts an LSP `textDocument/didChange` event into a [`Edit`] that
  /// can be consumed both by `ropey` and tree-sitter.
  fn build_edit<'a>(
    &self,
    change: &'a lsp::TextDocumentContentChangeEvent,
  ) -> Edit<'a> {
    let text = change.text.as_str();

    let text_end_bytes = text.len();

    let range = change.range.unwrap_or_else(|| lsp::Range {
      start: self.byte_to_lsp_position(0),
      end: self.byte_to_lsp_position(self.len_bytes()),
    });

    let (start, old_end) = (
      self.lsp_position_to_position(range.start),
      self.lsp_position_to_position(range.end),
    );

    let input_edit = InputEdit {
      new_end_byte: start.byte + text_end_bytes,
      new_end_position: start.point.advance(text.point_delta()),
      old_end_byte: old_end.byte,
      old_end_position: old_end.point,
      start_byte: start.byte,
      start_position: start.point,
    };

    Edit {
      end_char: old_end.char,
      input_edit,
      start_char: start.char,
      text,
    }
  }

  /// Maps an absolute byte offset into an LSP line/character pair where the
  /// column is expressed in UTF-16 code units as required by the spec.
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

  /// Converts an LSP position back into absolute byte/char offsets and a
  /// tree-sitter point so downstream consumers can choose whichever coordinate
  /// space they need.
  fn lsp_position_to_position(&self, position: lsp::Position) -> Position {
    let row_idx = position.line as usize;

    let row_char_idx = self.line_to_char(row_idx);
    let row_byte_idx = self.line_to_byte(row_idx);

    let col_char_idx = self.utf16_cu_to_char(
      self.char_to_utf16_cu(row_char_idx) + position.character as usize,
    );

    let col_byte_idx = self.char_to_byte(col_char_idx);

    Position {
      byte: col_byte_idx,
      char: col_char_idx,
      point: Point::new(row_idx, col_byte_idx - row_byte_idx),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq, ropey::Rope};

  type Range = (u32, u32, u32, u32);

  fn to_lsp_range(
    (start_line, start_character, end_line, end_character): Range,
  ) -> lsp::Range {
    lsp::Range {
      start: lsp::Position {
        line: start_line,
        character: start_character,
      },
      end: lsp::Position {
        line: end_line,
        character: end_character,
      },
    }
  }

  fn change(text: &str, range: Range) -> lsp::TextDocumentContentChangeEvent {
    lsp::TextDocumentContentChangeEvent {
      range: Some(to_lsp_range(range)),
      range_length: None,
      text: text.into(),
    }
  }

  #[test]
  fn apply_insert_into_empty_document() {
    let mut rope = Rope::from_str("");

    let change = change("🧪\nnew", (0, 0, 0, 0));

    let edit = rope.build_edit(&change);

    assert_eq!(
      edit,
      Edit {
        start_char: 0,
        end_char: 0,
        input_edit: InputEdit {
          start_byte: 0,
          old_end_byte: 0,
          new_end_byte: "🧪\nnew".len(),
          start_position: Point::new(0, 0),
          old_end_position: Point::new(0, 0),
          new_end_position: Point::new(1, 3),
        },
        text: "🧪\nnew",
      }
    );

    rope.apply_edit(&edit);

    assert_eq!(rope.to_string(), "🧪\nnew");
  }

  #[test]
  fn apply_insert_edit_updates_rope_contents() {
    let mut rope = Rope::from_str("hello world");

    let change = change("rope", (0, 6, 0, 11));

    let edit = rope.build_edit(&change);

    assert_eq!(
      edit,
      Edit {
        start_char: 6,
        end_char: 11,
        input_edit: InputEdit {
          new_end_byte: 10,
          new_end_position: Point::new(0, 10),
          old_end_byte: 11,
          old_end_position: Point::new(0, 11),
          start_byte: 6,
          start_position: Point::new(0, 6),
        },
        text: "rope",
      }
    );

    rope.apply_edit(&edit);

    assert_eq!(rope.to_string(), "hello rope");
  }

  #[test]
  fn apply_insert_edit_respects_utf16_columns() {
    let mut rope = Rope::from_str("ab");

    let change = change("🧪", (0, 1, 0, 1));

    let edit = rope.build_edit(&change);

    assert_eq!(
      edit,
      Edit {
        start_char: 1,
        end_char: 1,
        input_edit: InputEdit {
          new_end_byte: 5,
          new_end_position: Point::new(0, 5),
          old_end_byte: 1,
          old_end_position: Point::new(0, 1),
          start_byte: 1,
          start_position: Point::new(0, 1),
        },
        text: "🧪",
      }
    );

    rope.apply_edit(&edit);

    assert_eq!(rope.to_string(), "a🧪b");
  }

  #[test]
  fn apply_delete_edit_respects_utf16_columns() {
    let mut rope = Rope::from_str("a😊b");

    let change = change("", (0, 1, 0, 3));

    let edit = rope.build_edit(&change);

    assert_eq!(
      edit,
      Edit {
        start_char: 1,
        end_char: 2,
        input_edit: InputEdit {
          new_end_byte: 1,
          new_end_position: Point::new(0, 1),
          old_end_byte: 5,
          old_end_position: Point::new(0, 5),
          start_byte: 1,
          start_position: Point::new(0, 1),
        },
        text: "",
      }
    );

    rope.apply_edit(&edit);

    assert_eq!(rope.to_string(), "ab");
  }

  #[test]
  fn lsp_round_trip_handles_utf16_columns() {
    let rope = Rope::from_str("a😊b\nsecond");

    let position = rope.byte_to_lsp_position(5);

    assert_eq!(position, lsp::Position::new(0, 3));

    assert_eq!(
      rope.lsp_position_to_position(position),
      Position {
        byte: 5,
        char: 2,
        point: Point::new(0, 5),
      }
    );
  }

  #[test]
  fn replacement_across_surrogates_is_consistent() {
    let mut rope = Rope::from_str("foo😊bar");

    let change = change("🧪", (0, 3, 0, 5));

    let edit = rope.build_edit(&change);

    assert_eq!(
      edit,
      Edit {
        start_char: 3,
        end_char: 4,
        input_edit: InputEdit {
          start_byte: 3,
          old_end_byte: 7,
          new_end_byte: 7,
          start_position: Point::new(0, 3),
          old_end_position: Point::new(0, 7),
          new_end_position: Point::new(0, 7),
        },
        text: "🧪",
      }
    );

    rope.apply_edit(&edit);

    assert_eq!(rope.to_string(), "foo🧪bar");
  }

  #[test]
  fn multiline_edit_handles_utf16_offsets() {
    let mut rope = Rope::from_str("foo😊\nbar");

    let change = change("XX", (0, 2, 1, 1));

    let edit = rope.build_edit(&change);

    assert_eq!(
      edit,
      Edit {
        start_char: 2,
        end_char: 6,
        input_edit: InputEdit {
          start_byte: 2,
          old_end_byte: 9,
          new_end_byte: 4,
          start_position: Point::new(0, 2),
          old_end_position: Point::new(1, 1),
          new_end_position: Point::new(0, 4),
        },
        text: "XX",
      }
    );

    rope.apply_edit(&edit);

    assert_eq!(rope.to_string(), "foXXar");
  }

  #[test]
  fn append_beyond_eof_updates_point() {
    let mut rope = Rope::from_str("hi");

    let change = change("🧪\nnew", (0, 2, 0, 2));

    let edit = rope.build_edit(&change);

    assert_eq!(
      edit,
      Edit {
        start_char: 2,
        end_char: 2,
        input_edit: InputEdit {
          start_byte: 2,
          old_end_byte: 2,
          new_end_byte: 10,
          start_position: Point::new(0, 2),
          old_end_position: Point::new(0, 2),
          new_end_position: Point::new(1, 3),
        },
        text: "🧪\nnew",
      }
    );

    rope.apply_edit(&edit);

    assert_eq!(rope.to_string(), "hi🧪\nnew");
  }

  #[test]
  fn replace_entire_document_via_full_range() {
    let mut rope = Rope::from_str("foo😊bar");

    let change = lsp::TextDocumentContentChangeEvent {
      range: None,
      range_length: None,
      text: "🧪baz".into(),
    };

    let edit = rope.build_edit(&change);

    assert_eq!(
      edit,
      Edit {
        start_char: 0,
        end_char: 7,
        input_edit: InputEdit {
          start_byte: 0,
          old_end_byte: 10,
          new_end_byte: 7,
          start_position: Point::new(0, 0),
          old_end_position: Point::new(0, 10),
          new_end_position: Point::new(0, 7),
        },
        text: "🧪baz",
      }
    );

    rope.apply_edit(&edit);

    assert_eq!(rope.to_string(), "🧪baz");
  }
}
