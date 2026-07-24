use super::*;

pub trait StrExt {
  /// Returns the decoded value of a plain string literal.
  fn literal(&self) -> Option<String>;

  /// Returns a `Point` describing the tree-sitter point that would
  /// be reached after inserting this UTF-8 text.
  fn point_delta(&self) -> Point;
}

impl StrExt for str {
  fn literal(&self) -> Option<String> {
    let (quote, value) = if let Some(value) = self
      .strip_prefix('"')
      .and_then(|value| value.strip_suffix('"'))
    {
      ('"', value)
    } else {
      let value = self
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))?;

      ('\'', value)
    };

    if value.starts_with(quote) || value.ends_with(quote) {
      return None;
    }

    if quote == '\'' {
      return Some(value.to_string());
    }

    let mut cooked = String::new();

    let mut characters = value.chars();

    while let Some(character) = characters.next() {
      if character != '\\' {
        cooked.push(character);
        continue;
      }

      match characters.next()? {
        'n' => cooked.push('\n'),
        'r' => cooked.push('\r'),
        't' => cooked.push('\t'),
        '"' => cooked.push('"'),
        '\\' => cooked.push('\\'),
        '\n' => {}
        '\r' => {
          if characters.next()? != '\n' {
            return None;
          }
        }
        'u' => {
          if characters.next()? != '{' {
            return None;
          }

          let mut codepoint = String::new();

          loop {
            match characters.next()? {
              '}' => break,
              character if character.is_ascii_hexdigit() => {
                codepoint.push(character);
              }
              _ => return None,
            }
          }

          let codepoint = u32::from_str_radix(&codepoint, 16).ok()?;

          cooked.push(char::from_u32(codepoint)?);
        }
        _ => return None,
      }
    }

    Some(cooked)
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
  fn literal() {
    #[track_caller]
    fn case(source: &str, expected: Option<&str>) {
      assert_eq!(source.literal().as_deref(), expected);
    }

    case(r#""foo""#, Some("foo"));
    case(r#""\t""#, Some("\t"));
    case(r#""\u{2003}""#, Some("\u{2003}"));
    case(r"'\t'", Some("\\t"));
    case(r#"f"foo""#, None);
    case(r#"x"foo""#, None);
    case(r#"""foo""""#, None);
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
