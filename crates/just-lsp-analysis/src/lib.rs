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

#[derive(Clone, Debug)]
pub enum DocumentState {
  Error(String),
  Parsed(Arc<Document>),
}

impl PartialEq for DocumentState {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Parsed(lhs), Self::Parsed(rhs)) => Arc::ptr_eq(lhs, rhs),
      (Self::Error(lhs), Self::Error(rhs)) => lhs == rhs,
      _ => false,
    }
  }
}

impl Eq for DocumentState {}

#[salsa::tracked]
fn document_state(db: &dyn salsa::Database, file: SourceFile) -> DocumentState {
  match Document::from_text(
    file.uri(db).clone(),
    file.text(db).as_str(),
    file.version(db),
  ) {
    Ok(document) => DocumentState::Parsed(Arc::new(document)),
    Err(error) => DocumentState::Error(error.to_string()),
  }
}

#[salsa::tracked]
fn diagnostics(
  db: &dyn salsa::Database,
  file: SourceFile,
) -> Arc<Vec<lsp::Diagnostic>> {
  match document_state(db, file) {
    DocumentState::Parsed(document) => {
      Arc::new(Analyzer::new(&document).analyze())
    }
    DocumentState::Error(error) => Arc::new(vec![lsp::Diagnostic {
      message: format!("failed to parse `{}`: {error}", file.uri(db).as_str()),
      range: lsp::Range::default(),
      severity: Some(lsp::DiagnosticSeverity::ERROR),
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
  pub fn document(&self, file: SourceFile) -> Option<Arc<Document>> {
    match document_state(&self.db, file) {
      DocumentState::Parsed(document) => Some(document),
      DocumentState::Error(_) => None,
    }
  }

  #[must_use]
  pub fn file_text(&self, file: SourceFile) -> String {
    file.text(&self.db).clone()
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
