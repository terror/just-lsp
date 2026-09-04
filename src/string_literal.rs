use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StringLiteral {
  pub(crate) cooked: String,
  pub(crate) shell_expanded: bool,
}

impl StringLiteral {
  fn cook_string(text: &str) -> Option<String> {
    #[derive(PartialEq, Eq)]
    enum State {
      Backslash,
      BackslashCarriageReturn,
      Initial,
      Unicode,
      UnicodeValue { hex: String },
    }

    let mut cooked = String::new();

    let mut state = State::Initial;

    for c in text.chars() {
      match state {
        State::Initial => {
          if c == '\\' {
            state = State::Backslash;
          } else {
            cooked.push(c);
          }
        }
        State::Backslash if c == 'u' => {
          state = State::Unicode;
        }
        State::Backslash => {
          state = State::Initial;
          match c {
            'n' => cooked.push('\n'),
            'r' => cooked.push('\r'),
            't' => cooked.push('\t'),
            '\\' => cooked.push('\\'),
            '\n' => {}
            '\r' => state = State::BackslashCarriageReturn,
            '"' => cooked.push('"'),
            _ => return None,
          }
        }
        State::BackslashCarriageReturn => match c {
          '\n' => state = State::Initial,
          _ => return None,
        },
        State::Unicode => match c {
          '{' => {
            state = State::UnicodeValue { hex: String::new() };
          }
          _ => return None,
        },
        State::UnicodeValue { ref mut hex } => match c {
          '}' => {
            if hex.is_empty() {
              return None;
            }

            let codepoint = u32::from_str_radix(hex, 16).unwrap();

            cooked.push(char::from_u32(codepoint)?);

            state = State::Initial;
          }
          '0'..='9' | 'A'..='F' | 'a'..='f' => {
            hex.push(c);

            if hex.len() > 6 {
              return None;
            }
          }
          _ => return None,
        },
      }
    }

    match state {
      State::Initial => Some(cooked),
      _ => None,
    }
  }

  pub(crate) fn parse(source: &str) -> Option<Self> {
    let (shell_expanded, source) = source
      .strip_prefix('x')
      .map_or((false, source), |source| (true, source));

    let kind = StringKind::from_token_start(source)?;

    let delimiter = kind.delimiter();

    let raw = source.strip_prefix(delimiter)?.strip_suffix(delimiter)?;

    let uncooked = if kind.indented {
      Self::unindent(raw)
    } else {
      raw.to_owned()
    };

    let cooked = if kind.processes_escape_sequences() {
      Self::cook_string(&uncooked)?
    } else {
      uncooked
    };

    Some(Self {
      cooked,
      shell_expanded,
    })
  }

  fn unindent(source: &str) -> String {
    fn blank(line: &str) -> bool {
      line.trim_matches([' ', '\t', '\r', '\n']).is_empty()
    }

    fn common<'a>(left: &'a str, right: &str) -> &'a str {
      let length = left
        .bytes()
        .zip(right.bytes())
        .take_while(|(left, right)| left == right)
        .count();

      &left[..length]
    }

    fn indentation(line: &str) -> &str {
      &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
    }

    let lines = source.split_inclusive('\n').collect::<Vec<_>>();

    let common_indentation = lines
      .iter()
      .filter(|line| !blank(line))
      .copied()
      .map(indentation)
      .reduce(common)
      .unwrap_or("");

    let mut result = String::new();

    for (index, line) in lines.iter().enumerate() {
      let replacement =
        match (blank(line), index == 0, index == lines.len() - 1) {
          (true, false, false) if line.ends_with("\r\n") => "\r\n",
          (true, false, false) => "\n",
          (true, _, _) => "",
          (false, _, _) => &line[common_indentation.len()..],
        };

      result.push_str(replacement);
    }

    result
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cook_string() {
    #[track_caller]
    fn case(source: &str, expected: Option<&str>) {
      assert_eq!(StringLiteral::cook_string(source).as_deref(), expected);
    }

    case("", Some(""));
    case(r#"foo\n\r\t\\\"bar"#, Some("foo\n\r\t\\\"bar"));
    case("foo\\\n  bar", Some("foo  bar"));
    case("foo\\\r\nbar", Some("foobar"));
    case(r"\q", None);
    case("\\\rfoo", None);
    case("\\", None);
    case("\\\r", None);
    case(r"\u", None);
    case(r"\u{000066}oo", Some("foo"));
    case(r"\u{0000066}", None);
    case(r"\u{+66}", None);
    case(r"\u{D800}", None);
    case(r"\u{66", None);
    case(r"\u66", None);
    case(r"\u{}", None);
  }

  #[test]
  fn parse() {
    #[track_caller]
    fn case(source: &str, expected: Option<&str>) {
      assert_eq!(
        StringLiteral::parse(source)
          .map(|StringLiteral { cooked, .. }| cooked)
          .as_deref(),
        expected,
      );
    }

    case("''", Some(""));
    case("'''foo'''", Some("foo"));
    case("'''  foo'''", Some("foo"));
    case("'''foo\n  bar'''", Some("foo\n  bar"));
    case("'''\u{2003}foo'''", Some("\u{2003}foo"));
    case("\"\"\"\n  foo\n  bar\n\"\"\"", Some("foo\nbar\n"));
    case("\"\"\"\n  foo\\\n  bar\n\"\"\"", Some("foobar\n"));
    case("'''\n  foo\\t\n  bar\n'''", Some("foo\\t\nbar\n"));
    case("''''''", Some(""));
    case("''' \n\t'''", Some(""));
    case("'''\n  foo\n \n  bar\n  '''", Some("foo\n\nbar\n"));
    case(
      "'''\n  foo\r\n \t\r\n  bar\r\n  '''",
      Some("foo\r\n\r\nbar\r\n"),
    );
    case("'''\n \tfoo\n  bar\n'''", Some("\tfoo\n bar\n"));
    case("", None);
    case("'", None);
    case("'foo\"", None);
    case("''''", None);
    case(r#""foo\"""#, Some("foo\""));
    case(r#""""foo"bar""""#, Some("foo\"bar"));
    case("'''foo'bar'''", Some("foo'bar"));
  }

  #[test]
  fn parses_shell_expanded_strings() {
    let literal = StringLiteral::parse("x'foo'").unwrap();

    assert!(literal.shell_expanded);

    assert_eq!(literal.cooked, "foo");
  }
}
