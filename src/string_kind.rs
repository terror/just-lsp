use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StringKind {
  pub(crate) delimiter: StringDelimiter,
  pub(crate) indented: bool,
}

impl StringKind {
  const ALL: [Self; 4] = [
    Self {
      delimiter: StringDelimiter::QuoteDouble,
      indented: true,
    },
    Self {
      delimiter: StringDelimiter::QuoteSingle,
      indented: true,
    },
    Self {
      delimiter: StringDelimiter::QuoteDouble,
      indented: false,
    },
    Self {
      delimiter: StringDelimiter::QuoteSingle,
      indented: false,
    },
  ];

  pub(crate) fn delimiter(self) -> &'static str {
    match (self.delimiter, self.indented) {
      (StringDelimiter::QuoteDouble, false) => "\"",
      (StringDelimiter::QuoteDouble, true) => "\"\"\"",
      (StringDelimiter::QuoteSingle, false) => "'",
      (StringDelimiter::QuoteSingle, true) => "'''",
    }
  }

  pub(crate) fn from_token_start(source: &str) -> Option<Self> {
    Self::ALL
      .iter()
      .find(|kind| source.starts_with(kind.delimiter()))
      .copied()
  }

  pub(crate) fn processes_escape_sequences(self) -> bool {
    self.delimiter == StringDelimiter::QuoteDouble
  }
}
