use super::*;

#[derive(Debug)]
pub struct Diagnostic {
  /// A short header summarizing the diagnostic.
  pub display: String,
  /// A unique identifier for the diagnostic.
  pub id: String,
  /// A detailed message describing the diagnostic.
  pub message: String,
  /// The range in the source code where the diagnostic applies.
  pub range: lsp::Range,
  /// The severity level of the diagnostic.
  pub severity: lsp::DiagnosticSeverity,
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
      range,
      severity,
    }
  }

  pub fn warning(message: impl Into<String>, range: lsp::Range) -> Self {
    Self::new(message, range, lsp::DiagnosticSeverity::WARNING)
  }
}
