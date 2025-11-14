use super::*;

pub(crate) trait StrExt {
  /// Returns a `Point` describing the tree-sitter point that would
  /// be reached after inserting this UTF-8 text.
  fn point_delta(&self) -> Point;
}

impl StrExt for str {
  fn point_delta(&self) -> Point {
    let (mut rows, mut column) = (0usize, 0usize);

    let mut chars = self.chars().peekable();

    while let Some(ch) = chars.next() {
      match ch {
        '\r' => {
          if matches!(chars.peek().copied(), Some('\n')) {
            chars.next();
          }

          rows += 1;
          column = 0;
        }
        '\n' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}'
        | '\u{2029}' => {
          rows += 1;
          column = 0;
        }
        _ => {
          column += ch.len_utf8();
        }
      }
    }

    Point::new(rows, column)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn empty_string_produces_origin() {
    assert_eq!("".point_delta(), Point::new(0, 0));
  }

  #[test]
  fn ascii_text_advances_column_by_bytes() {
    assert_eq!("abc".point_delta(), Point::new(0, 3));
  }

  #[test]
  fn multibyte_chars_count_their_utf8_width() {
    assert_eq!("😊é".point_delta(), Point::new(0, "😊é".len()));
  }

  #[test]
  fn newline_moves_to_next_row_and_resets_column() {
    assert_eq!("hi\n😊".point_delta(), Point::new(1, "😊".len()));
  }

  #[test]
  fn crlf_sequences_count_as_single_newline() {
    assert_eq!("\r\nabc".point_delta(), Point::new(1, "abc".len()));
  }

  #[test]
  fn bare_carriage_return_counts_as_line_break() {
    assert_eq!("foo\rbar".point_delta(), Point::new(1, "bar".len()));
  }
}
