use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextPosition {
  pub(crate) byte: usize,
  pub(crate) char: usize,
  pub(crate) code: usize,
  pub(crate) point: tree_sitter::Point,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextEdit<'a> {
  pub(crate) end_char_idx: usize,
  pub(crate) input_edit: tree_sitter::InputEdit,
  pub(crate) start_char_idx: usize,
  pub(crate) text: &'a str,
}

pub trait RopeExt {
  fn apply_edit(&mut self, edit: &TextEdit);
  fn build_edit<'a>(
    &self,
    change: &'a lsp::TextDocumentContentChangeEvent,
  ) -> TextEdit<'a>;
  fn byte_to_lsp_position(&self, offset: usize) -> lsp::Position;
  fn byte_to_tree_sitter_point(&self, offset: usize) -> tree_sitter::Point;
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

      tree_sitter::Point::new(
        self.len_lines() + line_idx,
        text_end_byte_idx - line_byte_idx,
      )
    } else {
      self.byte_to_tree_sitter_point(new_end_byte)
    };

    let input_edit = tree_sitter::InputEdit {
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

    lsp::Position::new(line_idx as u32, character as u32)
  }

  fn byte_to_tree_sitter_point(&self, byte_idx: usize) -> tree_sitter::Point {
    let line_idx = self.byte_to_line(byte_idx);
    let line_byte_idx = self.line_to_byte(line_idx);
    tree_sitter::Point::new(line_idx, byte_idx - line_byte_idx)
  }

  fn lsp_position_to_core(&self, position: lsp::Position) -> TextPosition {
    let row_idx = position.line as usize;
    let col_code_idx = position.character as usize;

    let row_char_idx = self.line_to_char(row_idx);
    let col_char_idx = self.utf16_cu_to_char(col_code_idx);

    let row_byte_idx = self.line_to_byte(row_idx);
    let col_byte_idx = self.char_to_byte(col_char_idx);

    let row_code_idx = self.char_to_utf16_cu(row_char_idx);

    TextPosition {
      char: row_char_idx + col_char_idx,
      byte: row_byte_idx + col_byte_idx,
      code: row_code_idx + col_code_idx,
      point: tree_sitter::Point::new(row_idx, col_byte_idx),
    }
  }
}
