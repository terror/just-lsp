use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
  /// A short header summarizing the diagnostic.
  pub display: String,
  /// A unique identifier for the diagnostic.
  pub id: String,
  /// A detailed message describing the diagnostic.
  pub message: String,
  /// Quickfixes that can be applied to resolve the diagnostic.
  pub quickfixes: Vec<Quickfix>,
  /// The range in the source code where the diagnostic applies.
  pub range: lsp::Range,
  /// The severity level of the diagnostic.
  pub severity: lsp::DiagnosticSeverity,
}

impl Diagnostic {
  pub fn error(message: impl Into<String>, range: lsp::Range) -> Self {
    Self {
      display: String::new(),
      id: String::new(),
      message: message.into(),
      quickfixes: Vec::new(),
      range,
      severity: lsp::DiagnosticSeverity::ERROR,
    }
  }

  pub(crate) fn merge(analyses: Vec<Vec<Self>>) -> Vec<Self> {
    let mut quickfixes = analyses
      .first()
      .into_iter()
      .flatten()
      .flat_map(|diagnostic| diagnostic.quickfixes.iter().cloned())
      .collect::<Vec<_>>();

    for diagnostics in analyses.iter().skip(1) {
      quickfixes.retain(|quickfix| {
        diagnostics
          .iter()
          .any(|diagnostic| diagnostic.quickfixes.contains(quickfix))
      });
    }

    let mut diagnostics = Vec::<Self>::new();

    for mut diagnostic in analyses.into_iter().flatten() {
      diagnostic
        .quickfixes
        .retain(|quickfix| quickfixes.contains(quickfix));

      if let Some(previous) = diagnostics.iter_mut().find(|previous| {
        previous.id == diagnostic.id
          && previous.message == diagnostic.message
          && previous.range == diagnostic.range
          && previous.severity == diagnostic.severity
      }) {
        for quickfix in diagnostic.quickfixes {
          if !previous.quickfixes.contains(&quickfix) {
            previous.quickfixes.push(quickfix);
          }
        }
      } else {
        diagnostics.push(diagnostic);
      }
    }

    diagnostics.sort_by(|left, right| {
      left
        .range
        .start
        .cmp(&right.range.start)
        .then_with(|| left.message.cmp(&right.message))
    });

    diagnostics
  }

  #[must_use]
  pub fn quickfix(self, quickfix: impl Into<Option<Quickfix>>) -> Self {
    Self {
      quickfixes: self.quickfixes.into_iter().chain(quickfix.into()).collect(),
      ..self
    }
  }

  pub fn warning(message: impl Into<String>, range: lsp::Range) -> Self {
    Self {
      display: String::new(),
      id: String::new(),
      message: message.into(),
      quickfixes: Vec::new(),
      range,
      severity: lsp::DiagnosticSeverity::WARNING,
    }
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

impl PartialEq<lsp::Diagnostic> for Diagnostic {
  fn eq(&self, other: &lsp::Diagnostic) -> bool {
    lsp::Diagnostic::from(self.clone()).eq(other)
  }
}
