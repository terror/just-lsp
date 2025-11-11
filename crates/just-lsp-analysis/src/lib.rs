use {
  just_lsp_analyzer::Analyzer,
  just_lsp_document::Document,
  salsa::{Setter, Storage},
  std::sync::Arc,
  tower_lsp::lsp_types as lsp,
};

#[salsa::input]
pub struct SourceFile {
  #[returns(ref)]
  pub text: String,
  pub version: i32,
  #[returns(ref)]
  pub uri: lsp::Url,
}

pub type FileId = SourceFile;

#[salsa::tracked]
fn diagnostics(
  db: &dyn salsa::Database,
  file: SourceFile,
) -> Arc<Vec<lsp::Diagnostic>> {
  let (uri, text) = (file.uri(db), file.text(db));

  match Document::from_text(uri.clone(), text.as_str(), file.version(db)) {
    Ok(document) => Arc::new(Analyzer::new(&document).analyze()),
    Err(error) => Arc::new(vec![lsp::Diagnostic {
      range: lsp::Range::default(),
      severity: Some(lsp::DiagnosticSeverity::ERROR),
      message: format!("failed to parse `{uri}`: {error}"),
      ..Default::default()
    }]),
  }
}

#[salsa::db]
#[derive(Clone, Default)]
pub struct AnalysisDatabase {
  storage: Storage<Self>,
}

#[salsa::db]
impl salsa::Database for AnalysisDatabase {}

#[derive(Default)]
pub struct AnalysisHost {
  db: AnalysisDatabase,
}

impl AnalysisHost {
  pub fn clear_file(&mut self, file: SourceFile) {
    file.set_text(&mut self.db).to(String::new());
    file.set_version(&mut self.db).to(0);
  }

  #[must_use]
  pub fn diagnostics(&self, file: SourceFile) -> Arc<Vec<lsp::Diagnostic>> {
    diagnostics(&self.db, file)
  }

  #[must_use]
  pub fn file_version(&self, file: SourceFile) -> i32 {
    file.version(&self.db)
  }

  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  #[must_use]
  pub fn open_file(
    &mut self,
    uri: lsp::Url,
    text: String,
    version: i32,
  ) -> SourceFile {
    SourceFile::new(&self.db, text, version, uri)
  }

  pub fn update_file(&mut self, file: SourceFile, text: String, version: i32) {
    file.set_text(&mut self.db).to(text);
    file.set_version(&mut self.db).to(version);
  }
}
