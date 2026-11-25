use super::*;

define_rule! {
  InvalidUnicodeEscapeRule {
    id: "invalid-unicode-escape",
    title: "invalid unicode escape",
    severity: Error,
  }
}

impl InvalidUnicodeEscapeRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let document = context.document();

    context
      .local()
      .tree()
      .root_node()
      .find_all("escape_sequence")
      .into_iter()
      .filter_map(|escape| {
        let text = document.get_node_text(&escape);

        let hex = text.strip_prefix("\\u{")?.strip_suffix('}')?;

        let codepoint = u32::from_str_radix(hex, 16).ok()?;

        char::from_u32(codepoint).is_none().then(|| {
          Diagnostic::new(
            format!(
              "unicode escape sequence value `{hex}` greater than maximum valid code point `10FFFF`"
            ),
            document.get_range(&escape),
          )
        })
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn import_invalid_string_escape() {
    AnalyzerTest::new(r#"import "\u{D800}""#)
      .error(
        "unicode escape sequence value `D800` greater than maximum valid code point `10FFFF`",
        lsp::Range::at(0, 8, 0, 16),
      )
      .run();
  }

  #[test]
  fn invalid_unicode_escape() {
    #[track_caller]
    fn case(source: &str, errors: &[(&'static str, lsp::Range)]) {
      errors
        .iter()
        .fold(AnalyzerTest::new(source), |test, (message, range)| {
          test.error(message, *range)
        })
        .run();
    }

    case(r#"export foo := "\u{10FFFF}""#, &[]);

    case(
      r#"export foo := "\u{110000}""#,
      &[(
        "unicode escape sequence value `110000` greater than maximum valid code point `10FFFF`",
        lsp::Range::at(0, 15, 0, 25),
      )],
    );

    case(
      r#"export foo := "\u{d800}""#,
      &[(
        "unicode escape sequence value `d800` greater than maximum valid code point `10FFFF`",
        lsp::Range::at(0, 15, 0, 23),
      )],
    );
  }
}
