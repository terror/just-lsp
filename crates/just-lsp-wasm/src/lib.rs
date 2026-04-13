use {
  just_lsp::{Analyzer, Document},
  lsp_types::DiagnosticSeverity,
  serde::Serialize,
  wasm_bindgen::prelude::*,
};

#[derive(Serialize)]
struct Diagnostic {
  id: String,
  message: String,
  severity: &'static str,
  start_line: u32,
  start_character: u32,
  end_line: u32,
  end_character: u32,
}

#[wasm_bindgen]
pub fn analyze(source: &str) -> JsValue {
  serde_wasm_bindgen::to_value(
    &Analyzer::from(&Document::from(source))
      .analyze()
      .into_iter()
      .map(|diagnostic| Diagnostic {
        id: diagnostic.id,
        message: diagnostic.message,
        severity: match diagnostic.severity {
          DiagnosticSeverity::ERROR => "error",
          DiagnosticSeverity::WARNING => "warning",
          DiagnosticSeverity::INFORMATION => "information",
          DiagnosticSeverity::HINT => "hint",
          _ => "unknown",
        },
        start_line: diagnostic.range.start.line,
        start_character: diagnostic.range.start.character,
        end_line: diagnostic.range.end.line,
        end_character: diagnostic.range.end.character,
      })
      .collect::<Vec<_>>(),
  )
  .unwrap()
}
