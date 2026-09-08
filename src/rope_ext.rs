//! Extensions that bridge `ropey::Rope` with Language Server Protocol positions
//! and tree-sitter edit bookkeeping.
//!
//! The [`RopeExt`] trait is used inside `just-lsp` to keep three different
//! coordinate spaces (bytes, UTF-16 code units, and tree-sitter points) in sync
//! whenever an editor sends a `textDocument/didChange` notification.
//!
//! ```
//! use {
//!   just_lsp::RopeExt,
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
pub struct Position {
  pub byte: usize,
  pub char: usize,
  pub point: Point,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit<'a> {
  pub end_char: usize,
  pub input_edit: InputEdit,
  pub start_char: usize,
  pub text: &'a str,
}

pub trait RopeExt {
  fn apply_edit(&mut self, edit: &Edit);
  fn build_edit<'a>(
    &self,
    change: &'a lsp::TextDocumentContentChangeEvent,
  ) -> Edit<'a>;
  fn byte_to_lsp_position(&self, byte: usize) -> lsp::Position;
  fn lsp_position_to_position(&self, position: lsp::Position) -> Position;
}

impl RopeExt for Rope {
  /// Applies a previously constructed [`Edit`] to the rope, keeping both the
  /// textual contents and the internal tree-sitter offsets in sync.
  fn apply_edit(&mut self, edit: &Edit) {
    self.remove(edit.start_char..edit.end_char);

    if !edit.text.is_empty() {
      self.insert(edit.start_char, edit.text);
    }
  }

  /// Converts an LSP `textDocument/didChange` event into a [`Edit`] that can be
  /// consumed both by `ropey` and tree-sitter.
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
  fn byte_to_lsp_position(&self, byte: usize) -> lsp::Position {
    let line = self.byte_to_line(byte);

    let line_char = self.line_to_char(line);
    let line_utf16_cu = self.char_to_utf16_cu(line_char);

    let char = self.byte_to_char(byte);
    let char_utf16_cu = self.char_to_utf16_cu(char);

    lsp::Position::new(
      u32::try_from(line).expect("line index exceeds u32::MAX"),
      u32::try_from(char_utf16_cu - line_utf16_cu)
        .expect("character offset exceeds u32::MAX"),
    )
  }

  /// Converts an LSP position back into absolute byte/char offsets and a
  /// tree-sitter point so downstream consumers can choose whichever coordinate
  /// space they need.
  fn lsp_position_to_position(&self, position: lsp::Position) -> Position {
    let row = (position.line as usize).min(self.len_lines() - 1);

    let line = self.line(row);

    let column = if position.line as usize >= self.len_lines() {
      line.len_chars()
    } else {
      let line_break_len = line
        .chars_at(line.len_chars())
        .reversed()
        .take_while(|char| matches!(char, '\r' | '\n'))
        .count();

      line.utf16_cu_to_char(
        (position.character as usize).min(line.len_utf16_cu() - line_break_len),
      )
    };

    let char = self.line_to_char(row) + column;
    let byte = self.char_to_byte(char);

    Position {
      byte,
      char,
      point: Point::new(row, byte - self.line_to_byte(row)),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq, ropey::Rope};

  fn change(
    text: &str,
    range: lsp::Range,
  ) -> lsp::TextDocumentContentChangeEvent {
    lsp::TextDocumentContentChangeEvent {
      range: Some(range),
      range_length: None,
      text: text.into(),
    }
  }

  #[test]
  fn append_beyond_eof_updates_point() {
    let mut rope = Rope::from_str("hi");

    let change = change("🧪\nnew", lsp::Range::at(0, 2, 0, 2));

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
  fn apply_delete_edit_respects_utf16_columns() {
    let mut rope = Rope::from_str("a😊b");

    let change = change("", lsp::Range::at(0, 1, 0, 3));

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
  fn apply_insert_edit_respects_utf16_columns() {
    let mut rope = Rope::from_str("ab");

    let change = change("🧪", lsp::Range::at(0, 1, 0, 1));

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
  fn apply_insert_edit_updates_rope_contents() {
    let mut rope = Rope::from_str("hello world");

    let change = change("rope", lsp::Range::at(0, 6, 0, 11));

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
  fn apply_insert_into_empty_document() {
    let mut rope = Rope::from_str("");

    let change = change("🧪\nnew", lsp::Range::at(0, 0, 0, 0));

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
  fn edits_preserve_unicode_line_separators() {
    #[track_caller]
    fn case(separator: char) {
      let mut rope = Rope::from_str(&format!("foo{separator}bar\nqux"));

      let change = change("baz", lsp::Range::at(0, 4, 0, 7));

      let edit = rope.build_edit(&change);

      rope.apply_edit(&edit);

      assert_eq!(rope.to_string(), format!("foo{separator}baz\nqux"));

      let position = lsp::Position::new(1, 3);

      assert_eq!(rope.byte_to_lsp_position(rope.len_bytes()), position);

      assert_eq!(
        rope.lsp_position_to_position(position).byte,
        rope.len_bytes()
      );
    }

    for separator in
      ['\u{000B}', '\u{000C}', '\u{0085}', '\u{2028}', '\u{2029}']
    {
      case(separator);
    }
  }

  #[test]
  fn lsp_position_to_position_clamps_out_of_bounds_positions() {
    #[track_caller]
    fn case(source: &str, position: lsp::Position, expected: &Position) {
      assert_eq!(
        Rope::from_str(source).lsp_position_to_position(position),
        *expected
      );
    }

    for source in ["foo─🧪", "foo─🧪\nbar", "foo─🧪\r\nbar"] {
      for character in [6, 7, u32::MAX] {
        case(
          source,
          lsp::Position::new(0, character),
          &Position {
            byte: 10,
            char: 5,
            point: Point::new(0, 10),
          },
        );
      }
    }

    for position in [
      lsp::Position::new(1, u32::MAX),
      lsp::Position::new(2, 0),
      lsp::Position::new(u32::MAX, u32::MAX),
    ] {
      case(
        "🧪\nfoo─",
        position,
        &Position {
          byte: 11,
          char: 6,
          point: Point::new(1, 6),
        },
      );
    }

    for source in ["", "\nbar", "\r\nbar"] {
      case(
        source,
        lsp::Position::new(0, u32::MAX),
        &Position {
          byte: 0,
          char: 0,
          point: Point::new(0, 0),
        },
      );
    }

    case(
      "foo🧪\n",
      lsp::Position::new(u32::MAX, 0),
      &Position {
        byte: 8,
        char: 5,
        point: Point::new(1, 0),
      },
    );
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
  fn multiline_edit_handles_utf16_offsets() {
    let mut rope = Rope::from_str("foo😊\nbar");

    let change = change("XX", lsp::Range::at(0, 2, 1, 1));

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

  #[test]
  fn replacement_across_surrogates_is_consistent() {
    let mut rope = Rope::from_str("foo😊bar");

    let change = change("🧪", lsp::Range::at(0, 3, 0, 5));

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
}
