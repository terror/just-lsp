use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StringLiteral {
  pub(crate) cooked: String,
  kind: StringKind,
  pub(crate) shell_expanded: bool,
}

impl StringLiteral {
  fn cook(source: &str) -> Option<String> {
    let mut cooked = String::new();

    let mut characters = source.chars();

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

                if codepoint.len() > 6 {
                  return None;
                }
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

  pub(crate) fn parse(source: &str) -> Option<Self> {
    let (shell_expanded, source) = source
      .strip_prefix('x')
      .map_or((false, source), |source| (true, source));

    let kind = StringKind::from_token_start(source)?;

    let delimiter = kind.delimiter();

    if source.len() < delimiter.len() * 2 || !source.ends_with(delimiter) {
      return None;
    }

    let raw = &source[delimiter.len()..source.len() - delimiter.len()];

    let uncooked = if kind.indented {
      Self::unindent(raw)
    } else {
      raw.to_owned()
    };

    let cooked = if kind.processes_escape_sequences() {
      Self::cook(&uncooked)?
    } else {
      uncooked
    };

    Some(Self {
      cooked,
      kind,
      shell_expanded,
    })
  }

  pub(crate) fn parse_plain(source: &str) -> Option<Self> {
    let literal = Self::parse(source)?;

    if literal.kind.indented || literal.shell_expanded {
      None
    } else {
      Some(literal)
    }
  }

  fn unindent(source: &str) -> String {
    fn blank(line: &str) -> bool {
      line
        .chars()
        .all(|character| matches!(character, ' ' | '\t' | '\r' | '\n'))
    }

    fn common<'a>(left: &'a str, right: &str) -> &'a str {
      let length = left
        .char_indices()
        .zip(right.chars())
        .take_while(|((_, left), right)| left == right)
        .map(|((index, character), _)| index + character.len_utf8())
        .last()
        .unwrap_or(0);

      &left[..length]
    }

    fn indentation(line: &str) -> &str {
      let length = line
        .char_indices()
        .take_while(|(_, character)| matches!(character, ' ' | '\t'))
        .map(|(index, character)| index + character.len_utf8())
        .last()
        .unwrap_or(0);

      &line[..length]
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
      let replacement = match (
        blank(line),
        index == 0,
        index == lines.len().saturating_sub(1),
      ) {
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
  fn cooks_double_quoted_strings() {
    assert_eq!(
      StringLiteral::parse(r#""foo\t\u{2003}""#).unwrap().cooked,
      "foo\t\u{2003}",
    );
  }

  #[test]
  fn parses_indented_strings() {
    assert_eq!(
      StringLiteral::parse("\"\"\"\n  foo\n  bar\n\"\"\"")
        .unwrap()
        .cooked,
      "foo\nbar\n",
    );

    assert_eq!(StringLiteral::parse("'''foo'''").unwrap().cooked, "foo",);
  }

  #[test]
  fn parses_plain_strings() {
    assert_eq!(StringLiteral::parse("'foo'").unwrap().cooked, "foo");
    assert_eq!(StringLiteral::parse(r#""foo""#).unwrap().cooked, "foo");
  }

  #[test]
  fn parses_shell_expanded_strings() {
    let literal = StringLiteral::parse("x'foo'").unwrap();

    assert!(literal.shell_expanded);

    assert_eq!(literal.cooked, "foo");
  }

  #[test]
  fn plain_rejects_prefixed_and_indented_strings() {
    assert!(StringLiteral::parse_plain("x'foo'").is_none());
    assert!(StringLiteral::parse_plain("'''foo'''").is_none());
    assert!(StringLiteral::parse_plain("f'foo'").is_none());
  }
}
