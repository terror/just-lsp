use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct Diagnostic {
  /// A short header summarizing the diagnostic.
  pub display: String,
  /// A unique identifier for the diagnostic.
  pub id: String,
  /// A detailed message describing the diagnostic.
  pub message: String,
  /// An optional quickfix that can be applied to resolve the diagnostic.
  pub quickfix: Option<Quickfix>,
  /// The range in the source code where the diagnostic applies.
  pub range: lsp::Range,
  /// The severity level of the diagnostic.
  pub severity: lsp::DiagnosticSeverity,
}

impl Diagnostic {
  pub fn error(message: impl Into<String>, range: lsp::Range) -> Self {
    Self::new(message, range, lsp::DiagnosticSeverity::ERROR)
  }

  pub fn new(
    message: impl Into<String>,
    range: lsp::Range,
    severity: lsp::DiagnosticSeverity,
  ) -> Self {
    Self {
      display: String::new(),
      id: String::new(),
      message: message.into(),
      quickfix: None,
      range,
      severity,
    }
  }

  #[must_use]
  pub fn quickfix(self, quickfix: Quickfix) -> Self {
    Self {
      quickfix: Some(quickfix),
      ..self
    }
  }

  pub fn warning(message: impl Into<String>, range: lsp::Range) -> Self {
    Self::new(message, range, lsp::DiagnosticSeverity::WARNING)
  }
}

impl From<Diagnostic> for lsp::Diagnostic {
  fn from(value: Diagnostic) -> lsp::Diagnostic {
    lsp::Diagnostic {
      code: Some(lsp::NumberOrString::String(value.id)),
      message: value.message,
      range: value.range,
      severity: Some(value.severity),
      source: Some("just-lsp".to_string()),
      ..Default::default()
    }
  }
}
