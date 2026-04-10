use super::*;

const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");
const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");
const LOCALS_QUERY: &str = include_str!("../queries/locals.scm");

const TOKEN_TYPES: &[&str] = &[
  "comment",
  "keyword",
  "string",
  "operator",
  "variable",
  "parameter",
  "function",
  "namespace",
  "decorator",
  "boolean",
];

const TOKEN_MODIFIERS: &[&str] = &["declaration", "deprecated"];

const HIGHLIGHTS: &[(&str, Option<&str>, &[&str])] = &[
  ("keyword.import", Some("keyword"), &[]),
  ("keyword.conditional", Some("keyword"), &[]),
  ("keyword.directive", Some("keyword"), &[]),
  ("keyword", Some("keyword"), &[]),
  ("module", Some("namespace"), &[]),
  ("variable.parameter", Some("parameter"), &[]),
  ("variable", Some("variable"), &[]),
  ("function.call", Some("function"), &[]),
  ("function", Some("function"), &["declaration"]),
  ("attribute", Some("decorator"), &[]),
  ("operator", Some("operator"), &[]),
  ("punctuation.delimiter", Some("operator"), &[]),
  ("punctuation.bracket", Some("operator"), &[]),
  ("punctuation.special", Some("operator"), &[]),
  ("boolean", Some("boolean"), &[]),
  ("string.escape", Some("string"), &[]),
  ("string", Some("string"), &[]),
  ("comment", Some("comment"), &[]),
  ("spell", None, &[]),
  ("error", Some("keyword"), &["deprecated"]),
];

pub(crate) static SEMANTIC_TOKENS_LEGEND: LazyLock<lsp::SemanticTokensLegend> =
  LazyLock::new(|| lsp::SemanticTokensLegend {
    token_types: TOKEN_TYPES
      .iter()
      .map(|name| lsp::SemanticTokenType::new(name))
      .collect(),
    token_modifiers: TOKEN_MODIFIERS
      .iter()
      .map(|name| lsp::SemanticTokenModifier::new(name))
      .collect(),
  });

static HIGHLIGHT_CONFIGURATION: LazyLock<HighlightConfiguration> =
  LazyLock::new(|| {
    let mut configuration = HighlightConfiguration::new(
      // SAFETY: tree_sitter_just returns a valid static tree-sitter language.
      unsafe { tree_sitter_just() },
      "just",
      HIGHLIGHTS_QUERY,
      INJECTIONS_QUERY,
      LOCALS_QUERY,
    )
    .expect("Failed to create highlight configuration");

    let names = HIGHLIGHTS
      .iter()
      .map(|(name, _, _)| *name)
      .collect::<Vec<_>>();

    configuration.configure(&names);

    configuration
  });

static HIGHLIGHT_MAPPINGS: LazyLock<Vec<Option<TokenMap>>> =
  LazyLock::new(|| {
    HIGHLIGHTS
      .iter()
      .map(|(_, token_type, modifiers)| {
        token_type.map(|token_type| TokenMap::new(token_type, modifiers))
      })
      .collect()
  });

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Token {
  length: u32,
  line: u32,
  modifiers_bitset: u32,
  start_character: u32,
  token_type_index: u32,
}

#[derive(Clone, Copy)]
struct TokenMap {
  modifiers_bitset: u32,
  token_type_index: u32,
}

impl TokenMap {
  fn new(token_type: &str, modifiers: &[&str]) -> Self {
    Self {
      token_type_index: Tokenizer::token_type_index(token_type),
      modifiers_bitset: Tokenizer::modifier_bitset(modifiers),
    }
  }
}

pub(crate) struct Tokenizer<'doc> {
  document: &'doc Document,
}

impl<'doc> Tokenizer<'doc> {
  fn encode_tokens(mut tokens: Vec<Token>) -> Vec<lsp::SemanticToken> {
    tokens.sort_by(|left, right| {
      left
        .line
        .cmp(&right.line)
        .then(left.start_character.cmp(&right.start_character))
    });

    tokens
      .iter()
      .scan((0u32, 0u32), |(prev_line, prev_start), token| {
        let delta_line = token.line.saturating_sub(*prev_line);

        let delta_start = if delta_line > 0 {
          token.start_character
        } else {
          token.start_character.saturating_sub(*prev_start)
        };

        (*prev_line, *prev_start) = (token.line, token.start_character);

        Some(lsp::SemanticToken {
          delta_line,
          delta_start,
          length: token.length,
          token_modifiers_bitset: token.modifiers_bitset,
          token_type: token.token_type_index,
        })
      })
      .collect()
  }

  #[must_use]
  pub(crate) fn legend() -> &'static lsp::SemanticTokensLegend {
    &SEMANTIC_TOKENS_LEGEND
  }

  fn modifier_bitset(modifiers: &[&str]) -> u32 {
    modifiers
      .iter()
      .filter_map(|modifier| Self::modifier_index(modifier))
      .fold(0, |bitset, index| bitset | (1 << index))
  }

  fn modifier_index(modifier: &str) -> Option<u32> {
    TOKEN_MODIFIERS
      .iter()
      .position(|candidate| candidate == &modifier)
      .map(|index| {
        u32::try_from(index)
          .expect("Token modifier legend must fit within a u32")
      })
  }

  #[must_use]
  pub(crate) fn new(document: &'doc Document) -> Self {
    Self { document }
  }

  fn span_to_tokens(
    rope: &Rope,
    start_byte: usize,
    end_byte: usize,
    mapping: TokenMap,
  ) -> Vec<Token> {
    if start_byte >= end_byte {
      return Vec::new();
    }

    let (start_line, end_line) = (
      rope.byte_to_line(start_byte),
      rope.byte_to_line(end_byte.saturating_sub(1)),
    );

    (start_line..=end_line)
      .filter_map(|line_idx| {
        let line_start_byte = rope.line_to_byte(line_idx);

        let line_end_byte =
          rope.line_to_byte((line_idx + 1).min(rope.len_lines()));

        let segment_start = start_byte.max(line_start_byte);

        let segment_end = end_byte
          .min(
            line_end_byte
              .saturating_sub(Self::trailing_line_break_len(rope, line_idx)),
          )
          .max(segment_start);

        if segment_end <= segment_start {
          return None;
        }

        let line_utf16 = rope.char_to_utf16_cu(rope.line_to_char(line_idx));

        let start_utf16 =
          rope.char_to_utf16_cu(rope.byte_to_char(segment_start));

        let end_utf16 = rope.char_to_utf16_cu(rope.byte_to_char(segment_end));

        let length =
          u32::try_from(end_utf16.saturating_sub(start_utf16)).ok()?;

        if length == 0 {
          return None;
        }

        Some(Token {
          length,
          line: u32::try_from(line_idx).unwrap_or(0),
          modifiers_bitset: mapping.modifiers_bitset,
          start_character: u32::try_from(
            start_utf16.saturating_sub(line_utf16),
          )
          .ok()?,
          token_type_index: mapping.token_type_index,
        })
      })
      .collect()
  }

  fn token_type_index(token_type: &str) -> u32 {
    TOKEN_TYPES
      .iter()
      .position(|candidate| candidate == &token_type)
      .map(|index| {
        u32::try_from(index).expect("Token type legend must fit within a u32")
      })
      .expect("Token type missing from legend")
  }

  pub(crate) fn tokenize(&self) -> Result<Vec<lsp::SemanticToken>> {
    let mut highlighter = Highlighter::new();

    highlighter
      .parser()
      // SAFETY: tree_sitter_just returns a valid static tree-sitter language.
      .set_language(&unsafe { tree_sitter_just() })
      .map_err(|error| anyhow!("Failed to configure highlighter: {error}"))?;

    let source = self.document.content.to_string();

    let highlight_iter = highlighter
      .highlight(&HIGHLIGHT_CONFIGURATION, source.as_bytes(), None, |_| None)
      .map_err(|error| anyhow!("Failed to highlight document: {error}"))?;

    let mut highlight_stack = Vec::new();

    let tokens = highlight_iter
      .map(|event| {
        event.map_err(|error| anyhow!("Failed to highlight document: {error}"))
      })
      .filter_map(|event| match event {
        Err(error) => Some(Err(error)),
        Ok(HighlightEvent::HighlightStart(Highlight(index))) => {
          highlight_stack.push(index);
          None
        }
        Ok(HighlightEvent::HighlightEnd) => {
          highlight_stack.pop();
          None
        }
        Ok(HighlightEvent::Source { start, end }) => highlight_stack
          .iter()
          .rev()
          .find_map(|index| {
            HIGHLIGHT_MAPPINGS
              .get(*index)
              .and_then(|mapping| mapping.as_ref().copied())
          })
          .map(|mapping| {
            Ok(Self::span_to_tokens(
              &self.document.content,
              start,
              end,
              mapping,
            ))
          }),
      })
      .collect::<Result<Vec<_>>>()?
      .into_iter()
      .flatten()
      .collect();

    Ok(Self::encode_tokens(tokens))
  }

  fn trailing_line_break_len(rope: &Rope, line_idx: usize) -> usize {
    let line_start_byte = rope.line_to_byte(line_idx);

    let line_end_byte = rope.line_to_byte((line_idx + 1).min(rope.len_lines()));

    if line_end_byte <= line_start_byte {
      return 0;
    }

    if rope.byte(line_end_byte - 1) != b'\n' {
      return 0;
    }

    if line_end_byte - 1 > line_start_byte
      && rope.byte(line_end_byte - 2) == b'\r'
    {
      2
    } else {
      1
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  #[derive(Debug, PartialEq, Eq)]
  struct Expected {
    length: u32,
    line: u32,
    modifiers: u32,
    start: u32,
    token_type: &'static str,
  }

  fn to_expected(tokens: &[lsp::SemanticToken]) -> Vec<Expected> {
    let (mut line, mut start) = (0u32, 0u32);

    tokens
      .iter()
      .map(|token| {
        line += token.delta_line;

        start = if token.delta_line > 0 {
          token.delta_start
        } else {
          start + token.delta_start
        };

        Expected {
          length: token.length,
          line,
          modifiers: token.token_modifiers_bitset,
          start,
          token_type: TOKEN_TYPES[token.token_type as usize],
        }
      })
      .collect()
  }

  fn tokenize_to_expected(source: &str) -> Vec<Expected> {
    to_expected(&Tokenizer::new(&Document::from(source)).tokenize().unwrap())
  }

  #[test]
  fn recipe() {
    assert_eq!(
      tokenize_to_expected("foo:\n"),
      [
        Expected {
          line: 0,
          start: 0,
          length: 3,
          modifiers: Tokenizer::modifier_bitset(&["declaration"]),
          token_type: "function",
        },
        Expected {
          line: 0,
          start: 3,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
      ]
    );
  }

  #[test]
  fn assignment() {
    assert_eq!(
      tokenize_to_expected("foo := \"bar\"\n"),
      [
        Expected {
          line: 0,
          start: 0,
          length: 3,
          modifiers: 0,
          token_type: "variable",
        },
        Expected {
          line: 0,
          start: 4,
          length: 2,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 0,
          start: 7,
          length: 5,
          modifiers: 0,
          token_type: "string",
        },
      ]
    );
  }

  #[test]
  fn alias() {
    assert_eq!(
      tokenize_to_expected(indoc! {"
        foo:

        alias bar := foo
      "}),
      [
        Expected {
          line: 0,
          start: 0,
          length: 3,
          modifiers: Tokenizer::modifier_bitset(&["declaration"]),
          token_type: "function",
        },
        Expected {
          line: 0,
          start: 3,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 2,
          start: 0,
          length: 5,
          modifiers: 0,
          token_type: "keyword",
        },
        Expected {
          line: 2,
          start: 6,
          length: 3,
          modifiers: 0,
          token_type: "variable",
        },
        Expected {
          line: 2,
          start: 10,
          length: 2,
          modifiers: 0,
          token_type: "operator",
        },
      ]
    );
  }

  #[test]
  fn conditional() {
    assert_eq!(
      tokenize_to_expected(
        "foo := if \"a\" == \"b\" { \"c\" } else { \"d\" }\n"
      ),
      [
        Expected {
          line: 0,
          start: 0,
          length: 3,
          modifiers: 0,
          token_type: "variable",
        },
        Expected {
          line: 0,
          start: 4,
          length: 2,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 0,
          start: 7,
          length: 2,
          modifiers: 0,
          token_type: "keyword",
        },
        Expected {
          line: 0,
          start: 10,
          length: 3,
          modifiers: 0,
          token_type: "string",
        },
        Expected {
          line: 0,
          start: 14,
          length: 2,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 0,
          start: 17,
          length: 3,
          modifiers: 0,
          token_type: "string",
        },
        Expected {
          line: 0,
          start: 21,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 0,
          start: 23,
          length: 3,
          modifiers: 0,
          token_type: "string",
        },
        Expected {
          line: 0,
          start: 27,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 0,
          start: 29,
          length: 4,
          modifiers: 0,
          token_type: "keyword",
        },
        Expected {
          line: 0,
          start: 34,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 0,
          start: 36,
          length: 3,
          modifiers: 0,
          token_type: "string",
        },
        Expected {
          line: 0,
          start: 40,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
      ]
    );
  }

  #[test]
  fn function_call() {
    assert_eq!(
      tokenize_to_expected("foo := env(\"bar\")\n"),
      [
        Expected {
          line: 0,
          start: 0,
          length: 3,
          modifiers: 0,
          token_type: "variable",
        },
        Expected {
          line: 0,
          start: 4,
          length: 2,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 0,
          start: 7,
          length: 3,
          modifiers: 0,
          token_type: "function",
        },
        Expected {
          line: 0,
          start: 10,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 0,
          start: 11,
          length: 5,
          modifiers: 0,
          token_type: "string",
        },
        Expected {
          line: 0,
          start: 16,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
      ]
    );
  }

  #[test]
  fn comment() {
    assert_eq!(
      tokenize_to_expected("# foo\n"),
      [Expected {
        line: 0,
        start: 0,
        length: 5,
        modifiers: 0,
        token_type: "comment",
      }]
    );
  }

  #[test]
  fn boolean_setting() {
    assert_eq!(
      tokenize_to_expected("set export := true\n"),
      [
        Expected {
          line: 0,
          start: 0,
          length: 3,
          modifiers: 0,
          token_type: "keyword",
        },
        Expected {
          line: 0,
          start: 4,
          length: 6,
          modifiers: 0,
          token_type: "keyword",
        },
        Expected {
          line: 0,
          start: 11,
          length: 2,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 0,
          start: 14,
          length: 4,
          modifiers: 0,
          token_type: "boolean",
        },
      ]
    );
  }

  #[test]
  fn attribute() {
    assert_eq!(
      tokenize_to_expected(indoc! {"
        [private]
        foo:
      "}),
      [
        Expected {
          line: 0,
          start: 0,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 0,
          start: 1,
          length: 7,
          modifiers: 0,
          token_type: "decorator",
        },
        Expected {
          line: 0,
          start: 8,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 1,
          start: 0,
          length: 3,
          modifiers: Tokenizer::modifier_bitset(&["declaration"]),
          token_type: "function",
        },
        Expected {
          line: 1,
          start: 3,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
      ]
    );
  }

  #[test]
  fn parameters() {
    assert_eq!(
      tokenize_to_expected("foo bar baz:\n"),
      [
        Expected {
          line: 0,
          start: 0,
          length: 3,
          modifiers: Tokenizer::modifier_bitset(&["declaration"]),
          token_type: "function",
        },
        Expected {
          line: 0,
          start: 4,
          length: 3,
          modifiers: 0,
          token_type: "parameter",
        },
        Expected {
          line: 0,
          start: 8,
          length: 3,
          modifiers: 0,
          token_type: "parameter",
        },
        Expected {
          line: 0,
          start: 11,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
      ]
    );
  }

  #[test]
  fn dependencies() {
    assert_eq!(
      tokenize_to_expected(indoc! {"
        foo:
        bar: foo
      "}),
      [
        Expected {
          line: 0,
          start: 0,
          length: 3,
          modifiers: Tokenizer::modifier_bitset(&["declaration"]),
          token_type: "function",
        },
        Expected {
          line: 0,
          start: 3,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 1,
          start: 0,
          length: 3,
          modifiers: Tokenizer::modifier_bitset(&["declaration"]),
          token_type: "function",
        },
        Expected {
          line: 1,
          start: 3,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 1,
          start: 5,
          length: 3,
          modifiers: 0,
          token_type: "function",
        },
      ]
    );
  }

  #[test]
  fn import() {
    assert_eq!(
      tokenize_to_expected("import \"foo.just\"\n"),
      [
        Expected {
          line: 0,
          start: 0,
          length: 6,
          modifiers: 0,
          token_type: "keyword",
        },
        Expected {
          line: 0,
          start: 7,
          length: 10,
          modifiers: 0,
          token_type: "string",
        },
      ]
    );
  }

  #[test]
  fn module_declaration() {
    assert_eq!(
      tokenize_to_expected("mod foo\n"),
      [
        Expected {
          line: 0,
          start: 0,
          length: 3,
          modifiers: 0,
          token_type: "namespace",
        },
        Expected {
          line: 0,
          start: 4,
          length: 3,
          modifiers: 0,
          token_type: "namespace",
        },
      ]
    );
  }

  #[test]
  fn shebang() {
    assert_eq!(
      tokenize_to_expected(indoc! {"
        foo:
          #!/usr/bin/env bash
      "}),
      [
        Expected {
          line: 0,
          start: 0,
          length: 3,
          modifiers: Tokenizer::modifier_bitset(&["declaration"]),
          token_type: "function",
        },
        Expected {
          line: 0,
          start: 3,
          length: 1,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 1,
          start: 2,
          length: 19,
          modifiers: 0,
          token_type: "keyword",
        },
      ]
    );
  }

  #[test]
  fn export_assignment() {
    assert_eq!(
      tokenize_to_expected("export foo := \"bar\"\n"),
      [
        Expected {
          line: 0,
          start: 0,
          length: 6,
          modifiers: 0,
          token_type: "keyword",
        },
        Expected {
          line: 0,
          start: 7,
          length: 3,
          modifiers: 0,
          token_type: "variable",
        },
        Expected {
          line: 0,
          start: 11,
          length: 2,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 0,
          start: 14,
          length: 5,
          modifiers: 0,
          token_type: "string",
        },
      ]
    );
  }

  #[test]
  fn escape_sequence() {
    assert_eq!(
      tokenize_to_expected("foo := \"bar\\n\"\n"),
      [
        Expected {
          line: 0,
          start: 0,
          length: 3,
          modifiers: 0,
          token_type: "variable",
        },
        Expected {
          line: 0,
          start: 4,
          length: 2,
          modifiers: 0,
          token_type: "operator",
        },
        Expected {
          line: 0,
          start: 7,
          length: 4,
          modifiers: 0,
          token_type: "string",
        },
        Expected {
          line: 0,
          start: 11,
          length: 2,
          modifiers: 0,
          token_type: "string",
        },
        Expected {
          line: 0,
          start: 13,
          length: 1,
          modifiers: 0,
          token_type: "string",
        },
      ]
    );
  }

  #[test]
  fn empty_document() {
    assert_eq!(tokenize_to_expected(""), []);
  }

  #[test]
  fn multibyte_comment() {
    #[track_caller]
    fn case(source: &str) {
      let tokens = tokenize_to_expected(source);
      assert_eq!(tokens.len(), 1);
      assert_eq!(tokens[0].token_type, "comment");
      assert_eq!(tokens[0].line, 0);
      assert_eq!(tokens[0].start, 0);
    }

    case("# カ\n");
    case("# 😀\n");
    case("# å\n");
  }

  #[test]
  fn multiline_span() {
    let rope = Rope::from_str("foo\nbar\n");

    assert_eq!(
      Tokenizer::span_to_tokens(
        &rope,
        0,
        rope.len_bytes() - 1,
        TokenMap::new("keyword", &[]),
      ),
      [
        Token {
          length: 3,
          line: 0,
          modifiers_bitset: 0,
          start_character: 0,
          token_type_index: Tokenizer::token_type_index("keyword"),
        },
        Token {
          length: 3,
          line: 1,
          modifiers_bitset: 0,
          start_character: 0,
          token_type_index: Tokenizer::token_type_index("keyword"),
        },
      ]
    );
  }

  #[test]
  fn empty_span() {
    assert_eq!(
      Tokenizer::span_to_tokens(
        &Rope::from_str("foo"),
        0,
        0,
        TokenMap::new("keyword", &[]),
      ),
      []
    );
  }

  #[test]
  fn encode_sorts_and_computes_deltas() {
    let tokens = vec![
      Token {
        length: 2,
        line: 2,
        modifiers_bitset: 0,
        start_character: 1,
        token_type_index: 0,
      },
      Token {
        length: 3,
        line: 0,
        modifiers_bitset: 0,
        start_character: 5,
        token_type_index: 1,
      },
      Token {
        length: 4,
        line: 1,
        modifiers_bitset: 0,
        start_character: 0,
        token_type_index: 2,
      },
    ];

    assert_eq!(
      Tokenizer::encode_tokens(tokens),
      [
        lsp::SemanticToken {
          delta_line: 0,
          delta_start: 5,
          length: 3,
          token_modifiers_bitset: 0,
          token_type: 1,
        },
        lsp::SemanticToken {
          delta_line: 1,
          delta_start: 0,
          length: 4,
          token_modifiers_bitset: 0,
          token_type: 2,
        },
        lsp::SemanticToken {
          delta_line: 1,
          delta_start: 1,
          length: 2,
          token_modifiers_bitset: 0,
          token_type: 0,
        },
      ]
    );
  }

  #[test]
  fn encode_same_line_deltas() {
    let tokens = vec![
      Token {
        length: 3,
        line: 0,
        modifiers_bitset: 0,
        start_character: 0,
        token_type_index: 0,
      },
      Token {
        length: 2,
        line: 0,
        modifiers_bitset: 0,
        start_character: 5,
        token_type_index: 1,
      },
    ];

    assert_eq!(
      Tokenizer::encode_tokens(tokens),
      [
        lsp::SemanticToken {
          delta_line: 0,
          delta_start: 0,
          length: 3,
          token_modifiers_bitset: 0,
          token_type: 0,
        },
        lsp::SemanticToken {
          delta_line: 0,
          delta_start: 5,
          length: 2,
          token_modifiers_bitset: 0,
          token_type: 1,
        },
      ]
    );
  }

  #[test]
  fn encode_empty() {
    assert_eq!(Tokenizer::encode_tokens(Vec::new()), []);
  }

  #[test]
  fn token_type_indices() {
    #[track_caller]
    fn case(name: &str, expected: u32) {
      assert_eq!(Tokenizer::token_type_index(name), expected);
    }

    case("comment", 0);
    case("keyword", 1);
    case("string", 2);
    case("operator", 3);
    case("variable", 4);
    case("parameter", 5);
    case("function", 6);
    case("namespace", 7);
    case("decorator", 8);
    case("boolean", 9);
  }

  #[test]
  fn modifier_bitsets() {
    #[track_caller]
    fn case(modifiers: &[&str], expected: u32) {
      assert_eq!(Tokenizer::modifier_bitset(modifiers), expected);
    }

    case(&[], 0);
    case(&["declaration"], 1);
    case(&["deprecated"], 2);
    case(&["declaration", "deprecated"], 3);
  }

  #[test]
  fn trailing_line_break() {
    #[track_caller]
    fn case(s: &str, line: usize, expected: usize) {
      assert_eq!(
        Tokenizer::trailing_line_break_len(&Rope::from_str(s), line),
        expected
      );
    }

    case("foo\r\nbar", 0, 2);
    case("foo\r\nbar", 1, 0);
    case("foo\n", 0, 1);
    case("foo", 0, 0);
    case("# カ\n", 0, 1);
    case("# 😀\n", 0, 1);
    case("# å\n", 0, 1);
  }
}
