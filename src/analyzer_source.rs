use super::*;

#[derive(Debug)]
pub enum AnalyzerSource<'a> {
  Document {
    document: &'a Document,
    imported_documents: Vec<&'a Document>,
  },
  Project {
    documents: &'a DocumentStore,
    project: &'a Project,
  },
}
