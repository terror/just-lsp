use super::*;

#[derive(Debug)]
pub struct Project {
  pub dependencies: HashMap<lsp::Url, Vec<ProjectDependency>>,
  pub dependents: HashMap<lsp::Url, HashSet<lsp::Url>>,
  pub(super) imported: Vec<lsp::Url>,
  pub root: lsp::Url,
}

impl Project {
  pub fn imported_documents<'a>(
    &'a self,
    documents: &'a DocumentStore,
  ) -> impl Iterator<Item = &'a Document> {
    self.imported.iter().filter_map(|uri| documents.get(uri))
  }
}
