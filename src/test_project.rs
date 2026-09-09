use super::*;

#[derive(Debug)]
pub(super) struct TestProject {
  pub(super) document: Document,
  imported_documents: Vec<Document>,
}

impl TestProject {
  pub(super) fn imported_document(&mut self, content: &str) {
    let uri = lsp::Url::parse(&format!(
      "file:///foo{}.just",
      self.imported_documents.len()
    ))
    .unwrap();

    self
      .imported_documents
      .push(Document::new(content, uri).unwrap());
  }

  pub(super) fn new(document: Document) -> Self {
    Self {
      document,
      imported_documents: Vec::new(),
    }
  }

  pub(super) fn view(&self) -> ProjectView<'_> {
    ProjectView {
      document: &self.document,
      documents: once(&self.document)
        .chain(&self.imported_documents)
        .enumerate()
        .map(|(index, document)| ProjectViewDocument {
          document,
          load_depth: usize::from(index > 0),
        })
        .collect(),
    }
  }
}
