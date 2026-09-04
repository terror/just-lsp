use super::*;

pub trait StrExt {
  /// Returns the closest candidate within two edits.
  fn find_suggestion<'a>(
    &self,
    candidates: impl IntoIterator<Item = &'a str>,
  ) -> Option<String>;

  /// Returns a `Point` describing the tree-sitter point that would
  /// be reached after inserting this UTF-8 text.
  fn point_delta(&self) -> Point;
}

impl StrExt for str {
  fn find_suggestion<'a>(
    &self,
    candidates: impl IntoIterator<Item = &'a str>,
  ) -> Option<String> {
    candidates
      .into_iter()
      .map(|candidate| (strsim::levenshtein(self, candidate), candidate))
      .filter(|(distance, _)| *distance < 3)
      .min_by(|(left_distance, left), (right_distance, right)| {
        left_distance
          .cmp(right_distance)
          .then_with(|| left.cmp(right))
      })
      .map(|(_, candidate)| candidate.to_owned())
  }

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
  fn ascii_text_advances_column_by_bytes() {
    assert_eq!("abc".point_delta(), Point::new(0, 3));
  }

  #[test]
  fn bare_carriage_return_counts_as_line_break() {
    assert_eq!("foo\rbar".point_delta(), Point::new(1, "bar".len()));
  }

  #[test]
  fn crlf_sequences_count_as_single_newline() {
    assert_eq!("\r\nabc".point_delta(), Point::new(1, "abc".len()));
  }

  #[test]
  fn empty_string_produces_origin() {
    assert_eq!("".point_delta(), Point::new(0, 0));
  }

  #[test]
  fn find_suggestion_finds_close_candidate() {
    assert_eq!(
      "shel".find_suggestion(["shell", "export"]),
      Some("shell".into())
    );
  }

  #[test]
  fn find_suggestion_ignores_distant_candidates() {
    assert_eq!("unknown".find_suggestion(["shell", "export"]), None);
  }

  #[test]
  fn find_suggestion_resolves_ties_lexicographically() {
    assert_eq!("cat".find_suggestion(["bat", "car"]), Some("bat".into()));
  }

  #[test]
  fn multibyte_chars_count_their_utf8_width() {
    assert_eq!("😊é".point_delta(), Point::new(0, "😊é".len()));
  }

  #[test]
  fn newline_moves_to_next_row_and_resets_column() {
    assert_eq!("hi\n😊".point_delta(), Point::new(1, "😊".len()));
  }
}
