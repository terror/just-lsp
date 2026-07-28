use {
  just_lsp::{Analyzer, AnalyzerSource, Document},
  serde::Serialize,
  tower_lsp::lsp_types::DiagnosticSeverity,
  typeshare::typeshare,
  wasm_bindgen::prelude::*,
};

#[derive(Serialize)]
#[typeshare]
struct Diagnostic {
  end_character: u32,
  end_line: u32,
  id: String,
  message: String,
  severity: &'static str,
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
      source: AnalyzerSource::Document {
        document: &document,
        imported_documents: Vec::new(),
      },
    }
    .analyze()
    .into_iter()
    .map(|diagnostic| {
      let diagnostic = diagnostic.value;

      Diagnostic {
        end_character: diagnostic.range.end.character,
        end_line: diagnostic.range.end.line,
        id: diagnostic.id,
        message: diagnostic.message,
        severity: match diagnostic.severity {
          DiagnosticSeverity::ERROR => "error",
          DiagnosticSeverity::HINT => "hint",
          DiagnosticSeverity::INFORMATION => "information",
          DiagnosticSeverity::WARNING => "warning",
          _ => "unknown",
        },
        start_character: diagnostic.range.start.character,
        start_line: diagnostic.range.start.line,
      }
    })
    .collect::<Vec<_>>(),
  )
  .map_err(|error| JsError::new(&error.to_string()))
}
