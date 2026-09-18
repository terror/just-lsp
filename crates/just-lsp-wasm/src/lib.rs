use {
  just_lsp::{Analyzer, Document, Resolver},
  serde::Serialize,
  tower_lsp::lsp_types::{
    DiagnosticSeverity, HoverContents, MarkupKind, Position,
  },
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
struct Hover {
  content: String,
  end_character: u32,
  end_line: u32,
  markdown: bool,
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

/// # Errors
///
/// Returns a `JsError` if serialization of hover information fails.
#[wasm_bindgen]
pub fn hover(
  source: &str,
  line: u32,
  character: u32,
) -> Result<JsValue, JsError> {
  let document = Document::from(source);

  let hover = document
    .node_at_position(Position::new(line, character))
    .filter(|node| node.kind() == "identifier")
    .and_then(|identifier| {
      Resolver::new(&document).resolve_identifier_hover(&identifier)
    })
    .and_then(|hover| {
      let HoverContents::Markup(contents) = hover.contents else {
        return None;
      };

      let range = hover.range?;

      Some(Hover {
        content: contents.value,
        end_character: range.end.character,
        end_line: range.end.line,
        markdown: contents.kind == MarkupKind::Markdown,
        start_character: range.start.character,
        start_line: range.start.line,
      })
    });

  serde_wasm_bindgen::to_value(&hover)
    .map_err(|error| JsError::new(&error.to_string()))
}
