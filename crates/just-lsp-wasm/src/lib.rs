use {
  just_lsp::{Analyzer, Document},
  serde::Serialize,
  tower_lsp::lsp_types::DiagnosticSeverity,
  typeshare_annotation::typeshare,
  wasm_bindgen::prelude::*,
};

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
#[typeshare]
enum Severity {
  Error,
  Hint,
  Info,
  Warning,
}

impl From<DiagnosticSeverity> for Severity {
  fn from(severity: DiagnosticSeverity) -> Self {
    match severity {
      DiagnosticSeverity::ERROR => Self::Error,
      DiagnosticSeverity::HINT => Self::Hint,
      DiagnosticSeverity::WARNING => Self::Warning,
      _ => Self::Info,
    }
  }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
struct Diagnostic {
  end_character: u32,
  end_line: u32,
  id: String,
  message: String,
  severity: Severity,
  start_character: u32,
  start_line: u32,
}

/// # Errors
///
/// Returns a `JsError` if serialization of diagnostics fails.
#[wasm_bindgen]
pub fn analyze(source: &str) -> Result<JsValue, JsError> {
  let document = Document::from(source);

  serde_wasm_bindgen::to_value(
    &Analyzer {
      config: None,
      view: (&document).into(),
    }
    .analyze()
    .into_iter()
    .map(|diagnostic| Diagnostic {
      end_character: diagnostic.range.end.character,
      end_line: diagnostic.range.end.line,
      id: diagnostic.id,
      message: diagnostic.message,
      severity: diagnostic.severity.into(),
      start_character: diagnostic.range.start.character,
      start_line: diagnostic.range.start.line,
    })
    .collect::<Vec<_>>(),
  )
  .map_err(|error| JsError::new(&error.to_string()))
}
