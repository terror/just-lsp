use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IndentKind {
  Mixed,
  Spaces,
  Tabs,
}

#[derive(Debug)]
pub(crate) struct RecipeLine<'a> {
  pub continues: bool,
  pub indent: &'a str,
  pub kind: IndentKind,
  pub number: u32,
}

impl<'a> RecipeLine<'a> {
  pub(crate) fn parse(line: &'a TextNode) -> Option<Self> {
    let content = line.value.as_str();

    if content.trim().is_empty() {
      return None;
    }

    let indent =
      &content[..content.len() - content.trim_start_matches([' ', '\t']).len()];

    let kind = match (indent.contains(' '), indent.contains('\t')) {
      (true, true) => IndentKind::Mixed,
      (true, false) => IndentKind::Spaces,
      (false, true) => IndentKind::Tabs,
      (false, false) => return None,
    };

    Some(Self {
      continues: content.trim_end().ends_with('\\'),
      indent,
      kind,
      number: line.range.start.line,
    })
  }

  pub(crate) fn range(&self) -> lsp::Range {
    lsp::Range::at(
      self.number,
      0,
      self.number,
      u32::try_from(self.indent.len()).unwrap_or(u32::MAX),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    #[track_caller]
    fn case(source: &str, expected: Option<(&str, IndentKind, bool)>) {
      let line = TextNode {
        value: source.into(),
        range: lsp::Range::default(),
      };

      assert_eq!(
        RecipeLine::parse(&line).map(|line| (
          line.indent,
          line.kind,
          line.continues
        )),
        expected,
      );
    }

    case("", None);
    case(" \t\r", None);
    case("foo", None);
    case("  foo", Some(("  ", IndentKind::Spaces, false)));
    case("\tfoo", Some(("\t", IndentKind::Tabs, false)));
    case(" \tfoo", Some((" \t", IndentKind::Mixed, false)));
    case("\t foo \\ \r", Some(("\t ", IndentKind::Mixed, true)));
  }
}
