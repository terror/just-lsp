use super::*;

const HIGHLIGHTS_QUERY: &str =
  include_str!("../vendor/tree-sitter-just/queries/just/highlights.scm");

const INJECTIONS_QUERY: &str =
  include_str!("../vendor/tree-sitter-just/queries/just/injections.scm");

const LOCALS_QUERY: &str =
  include_str!("../vendor/tree-sitter-just/queries/just/locals.scm");

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

const HIGHLIGHT_NAMES: &[&str] = &[
  "keyword.import",
  "keyword.conditional",
  "keyword.directive",
  "keyword",
  "module",
  "variable.parameter",
  "variable",
  "function.call",
  "function",
  "attribute",
  "operator",
  "punctuation.delimiter",
  "punctuation.bracket",
  "punctuation.special",
  "boolean",
  "string.escape",
  "string",
  "comment",
  "spell",
  "error",
];

pub(crate) static SEMANTIC_TOKENS_LEGEND: Lazy<lsp::SemanticTokensLegend> =
  Lazy::new(|| lsp::SemanticTokensLegend {
    token_types: TOKEN_TYPES
      .iter()
      .map(|name| lsp::SemanticTokenType::new(name))
      .collect(),
    token_modifiers: TOKEN_MODIFIERS
      .iter()
      .map(|name| lsp::SemanticTokenModifier::new(name))
      .collect(),
  });

static HIGHLIGHT_CONFIGURATION: Lazy<HighlightConfiguration> =
  Lazy::new(|| {
    let mut configuration = HighlightConfiguration::new(
      // SAFETY: tree_sitter_just exposes a static tree-sitter language definition.
      unsafe { tree_sitter_just() },
      "just",
      HIGHLIGHTS_QUERY,
      INJECTIONS_QUERY,
      LOCALS_QUERY,
    )
    .expect("Failed to create highlight configuration");

    configuration.configure(HIGHLIGHT_NAMES);

    configuration
  });

static HIGHLIGHT_MAPPINGS: Lazy<Vec<Option<SemanticTokenMapping>>> =
  Lazy::new(|| {
    HIGHLIGHT_NAMES
      .iter()
      .map(|name| match *name {
        "keyword"
        | "keyword.conditional"
        | "keyword.directive"
        | "keyword.import" => Some(SemanticTokenMapping::new("keyword", &[])),
        "module" => Some(SemanticTokenMapping::new("namespace", &[])),
        "variable" => Some(SemanticTokenMapping::new("variable", &[])),
        "variable.parameter" => {
          Some(SemanticTokenMapping::new("parameter", &[]))
        }
        "function" => {
          Some(SemanticTokenMapping::new("function", &["declaration"]))
        }
        "function.call" => Some(SemanticTokenMapping::new("function", &[])),
        "attribute" => Some(SemanticTokenMapping::new("decorator", &[])),
        "operator"
        | "punctuation.delimiter"
        | "punctuation.bracket"
        | "punctuation.special" => {
          Some(SemanticTokenMapping::new("operator", &[]))
        }
        "boolean" => Some(SemanticTokenMapping::new("boolean", &[])),
        "string" | "string.escape" => {
          Some(SemanticTokenMapping::new("string", &[]))
        }
        "comment" => Some(SemanticTokenMapping::new("comment", &[])),
        "error" => Some(SemanticTokenMapping::new("keyword", &["deprecated"])),
        _ => None,
      })
      .collect()
  });

#[derive(Clone, Copy)]
struct SemanticTokenMapping {
  modifiers_bitset: u32,
  token_type_index: u32,
}

impl SemanticTokenMapping {
  fn new(token_type: &str, modifiers: &[&str]) -> Self {
    Self {
      token_type_index: Tokenizer::token_type_index(token_type),
      modifiers_bitset: Tokenizer::modifier_bitset(modifiers),
    }
  }
}

struct TokenData {
  length: u32,
  line: u32,
  modifiers_bitset: u32,
  start_character: u32,
  token_type_index: u32,
}

pub(crate) struct Tokenizer<'doc> {
  document: &'doc Document,
}

impl<'doc> Tokenizer<'doc> {
  fn char_at_byte(rope: &Rope, byte_idx: usize) -> Option<char> {
    if byte_idx >= rope.len_bytes() {
      return None;
    }

    let end = (byte_idx + 4).min(rope.len_bytes());

    rope.byte_slice(byte_idx..end).chars().next()
  }
  fn encode_tokens(mut tokens: Vec<TokenData>) -> Vec<lsp::SemanticToken> {
    tokens.sort_by(|left, right| {
      left
        .line
        .cmp(&right.line)
        .then(left.start_character.cmp(&right.start_character))
    });

    let mut data = Vec::with_capacity(tokens.len());

    let mut previous_line = 0;
    let mut previous_start = 0;
    let mut first = true;

    for token in tokens {
      let delta_line = if first {
        token.line
      } else {
        token.line.saturating_sub(previous_line)
      };

      let delta_start = if first || delta_line > 0 {
        token.start_character
      } else {
        token.start_character.saturating_sub(previous_start)
      };

      data.push(lsp::SemanticToken {
        delta_line,
        delta_start,
        length: token.length,
        token_type: token.token_type_index,
        token_modifiers_bitset: token.modifiers_bitset,
      });

      previous_line = token.line;
      previous_start = token.start_character;
      first = false;
    }

    data
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

  fn push_tokens_for_span(
    rope: &Rope,
    start_byte: usize,
    end_byte: usize,
    mapping: SemanticTokenMapping,
    tokens: &mut Vec<TokenData>,
  ) {
    if start_byte >= end_byte {
      return;
    }

    let last_byte = end_byte.saturating_sub(1);

    let start_line = rope.byte_to_line(start_byte);
    let end_line = rope.byte_to_line(last_byte);

    for line_idx in start_line..=end_line {
      let line_start_byte = rope.line_to_byte(line_idx);
      let next_line_idx = (line_idx + 1).min(rope.len_lines());
      let line_end_byte = rope.line_to_byte(next_line_idx);

      let segment_start = if line_idx == start_line {
        start_byte
      } else {
        line_start_byte
      };

      let mut segment_end = if line_idx == end_line {
        end_byte
      } else {
        let newline_len = Self::trailing_line_break_len(rope, line_idx);
        line_end_byte.saturating_sub(newline_len)
      };

      if segment_end > end_byte {
        segment_end = end_byte;
      }

      if segment_end <= segment_start {
        continue;
      }

      let line_char_idx = rope.line_to_char(line_idx);
      let start_char_idx = rope.byte_to_char(segment_start);
      let end_char_idx = rope.byte_to_char(segment_end);

      let line_utf16 = rope.char_to_utf16_cu(line_char_idx);
      let start_utf16 = rope.char_to_utf16_cu(start_char_idx);
      let end_utf16 = rope.char_to_utf16_cu(end_char_idx);

      let start_character = start_utf16.saturating_sub(line_utf16);
      let end_character = end_utf16.saturating_sub(line_utf16);

      if end_character <= start_character {
        continue;
      }

      let length = end_character - start_character;

      let Ok(line) = u32::try_from(line_idx) else {
        continue;
      };

      let Ok(start_character) = u32::try_from(start_character) else {
        continue;
      };

      let Ok(length) = u32::try_from(length) else {
        continue;
      };

      tokens.push(TokenData {
        length,
        line,
        modifiers_bitset: mapping.modifiers_bitset,
        start_character,
        token_type_index: mapping.token_type_index,
      });
    }
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
      .set_language(
        // SAFETY: The generated parser exposes a valid static tree-sitter language.
        &unsafe { tree_sitter_just() },
      )
      .map_err(|error| anyhow!("Failed to configure highlighter: {error}"))?;

    let source = self.document.content.to_string();

    let highlight_iter = highlighter
      .highlight(&HIGHLIGHT_CONFIGURATION, source.as_bytes(), None, |_| None)
      .map_err(|error| anyhow!("Failed to highlight document: {error}"))?;

    let mut highlight_stack = Vec::new();
    let mut tokens = Vec::new();

    for event in highlight_iter {
      match event
        .map_err(|error| anyhow!("Failed to highlight document: {error}"))?
      {
        HighlightEvent::HighlightStart(Highlight(index)) => {
          highlight_stack.push(index);
        }
        HighlightEvent::HighlightEnd => {
          highlight_stack.pop();
        }
        HighlightEvent::Source { start, end } => {
          if start >= end {
            continue;
          }

          if let Some(mapping) =
            highlight_stack.iter().rev().find_map(|index| {
              HIGHLIGHT_MAPPINGS
                .get(*index)
                .and_then(|mapping| mapping.as_ref().copied())
            })
          {
            Self::push_tokens_for_span(
              &self.document.content,
              start,
              end,
              mapping,
              &mut tokens,
            );
          }
        }
      }
    }

    Ok(Self::encode_tokens(tokens))
  }

  fn trailing_line_break_len(rope: &Rope, line_idx: usize) -> usize {
    let line_start_byte = rope.line_to_byte(line_idx);
    let line_end_byte = rope.line_to_byte((line_idx + 1).min(rope.len_lines()));

    if line_end_byte <= line_start_byte || line_end_byte == 0 {
      return 0;
    }

    let last_byte = line_end_byte - 1;

    let Some(last_char) = Self::char_at_byte(rope, last_byte) else {
      return 0;
    };

    if last_char != '\n' {
      return 0;
    }

    if last_byte == 0 {
      return 1;
    }

    if last_byte <= line_start_byte {
      return 1;
    }

    if matches!(Self::char_at_byte(rope, last_byte - 1), Some('\r')) {
      2
    } else {
      1
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn document(text: &str) -> Document {
    let params = lsp::DidOpenTextDocumentParams {
      text_document: lsp::TextDocumentItem {
        uri: lsp::Url::parse("file:///tokenizer.just").unwrap(),
        language_id: "just".into(),
        version: 1,
        text: text.into(),
      },
    };

    Document::try_from(params).expect("document creation succeeds")
  }

  #[test]
  fn token_type_index_matches_expected_order() {
    assert_eq!(Tokenizer::token_type_index("comment"), 0);
    assert_eq!(Tokenizer::token_type_index("keyword"), 1);
    assert_eq!(Tokenizer::token_type_index("function"), 6);
  }

  #[test]
  fn modifier_bitset_combines_flags() {
    assert_eq!(Tokenizer::modifier_bitset(&["declaration"]), 1);
    assert_eq!(Tokenizer::modifier_bitset(&["deprecated"]), 2);
    assert_eq!(
      Tokenizer::modifier_bitset(&["declaration", "deprecated"]),
      3
    );
  }

  #[test]
  fn push_tokens_for_span_handles_multiline_segments() {
    let text = "alpha\nbeta\n";
    let rope = Rope::from_str(text);
    let mapping = SemanticTokenMapping::new("keyword", &[]);
    let mut tokens = Vec::new();

    let end_byte = text.len() - 1;

    Tokenizer::push_tokens_for_span(&rope, 0, end_byte, mapping, &mut tokens);

    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].line, 0);
    assert_eq!(tokens[0].start_character, 0);
    assert_eq!(tokens[0].length, 5);
    assert_eq!(tokens[1].line, 1);
    assert_eq!(tokens[1].start_character, 0);
    assert_eq!(tokens[1].length, 4);
  }

  #[test]
  fn encode_tokens_sorts_and_computes_deltas() {
    let tokens = vec![
      TokenData {
        line: 2,
        start_character: 1,
        length: 2,
        token_type_index: 0,
        modifiers_bitset: 0,
      },
      TokenData {
        line: 0,
        start_character: 5,
        length: 3,
        token_type_index: 1,
        modifiers_bitset: 0,
      },
      TokenData {
        line: 1,
        start_character: 0,
        length: 4,
        token_type_index: 2,
        modifiers_bitset: 0,
      },
    ];

    let encoded = Tokenizer::encode_tokens(tokens);

    assert_eq!(encoded.len(), 3);
    assert_eq!(encoded[0].delta_line, 0);
    assert_eq!(encoded[0].delta_start, 5);
    assert_eq!(encoded[1].delta_line, 1);
    assert_eq!(encoded[1].delta_start, 0);
    assert_eq!(encoded[2].delta_line, 1);
    assert_eq!(encoded[2].delta_start, 1);
  }

  #[test]
  fn trailing_line_break_len_detects_crlf() {
    let rope = Rope::from_str("foo\r\nbar");
    assert_eq!(Tokenizer::trailing_line_break_len(&rope, 0), 2);
    assert_eq!(Tokenizer::trailing_line_break_len(&rope, 1), 0);
  }

  #[test]
  fn tokenizer_emits_expected_tokens() -> Result {
    let document = document("foo:\n  echo \"bar\"\n");

    let tokens = Tokenizer::new(&document).tokenize()?;

    assert_eq!(
      tokens,
      vec![
        lsp::SemanticToken {
          delta_line: 0,
          delta_start: 0,
          length: 3,
          token_type: Tokenizer::token_type_index("function"),
          token_modifiers_bitset: Tokenizer::modifier_bitset(&["declaration"]),
        },
        lsp::SemanticToken {
          delta_line: 0,
          delta_start: 3,
          length: 1,
          token_type: Tokenizer::token_type_index("operator"),
          token_modifiers_bitset: 0,
        },
      ],
    );

    Ok(())
  }
}
