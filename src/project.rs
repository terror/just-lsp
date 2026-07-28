use super::*;

#[derive(Debug)]
pub struct Project {
  pub dependencies: HashMap<lsp::Url, Vec<ProjectDependency>>,
  pub dependents: HashMap<lsp::Url, HashSet<lsp::Url>>,
  pub(super) imported: Vec<lsp::Url>,
  pub root: lsp::Url,
}

impl Project {
  pub(super) fn add_dependency(
    &mut self,
    source: &lsp::Url,
    dependency: ProjectDependency,
  ) {
    self
      .dependencies
      .entry(source.clone())
      .or_default()
      .push(dependency);
  }

  pub(super) fn add_dependent(
    &mut self,
    dependency: &lsp::Url,
    source: &lsp::Url,
  ) {
    self
      .dependents
      .entry(dependency.clone())
      .or_default()
      .insert(source.clone());
  }

  pub fn dependencies(
    &self,
    source: &lsp::Url,
  ) -> impl Iterator<Item = &ProjectDependency> {
    self.dependencies.get(source).into_iter().flatten()
  }

  #[must_use]
  pub fn dependency_at(
    &self,
    source: &lsp::Url,
    position: lsp::Position,
  ) -> Option<&ProjectDependency> {
    self.dependencies(source).find(|dependency| {
      dependency
        .location
        .overlaps(lsp::Range::new(position, position))
    })
  }

  pub fn imported_documents<'a>(
    &'a self,
    documents: &'a DocumentStore,
  ) -> impl Iterator<Item = &'a Document> {
    self.imported.iter().filter_map(|uri| documents.get(uri))
  }

  #[must_use]
  pub fn new(root: lsp::Url) -> Self {
    Self {
      dependencies: HashMap::new(),
      dependents: HashMap::new(),
      imported: Vec::new(),
      root,
    }
  }
}
