use super::*;

#[derive(Debug)]
pub(crate) struct Project {
  pub(crate) dependencies: HashMap<lsp::Url, Vec<ProjectDependency>>,
  pub(crate) import_scope: ImportScope,
  pub(crate) root: lsp::Url,
  pub(crate) targets: HashSet<lsp::Url>,
}

impl Project {
  #[must_use]
  pub(crate) fn contains(&self, uri: &lsp::Url) -> bool {
    self.root == *uri || self.targets.contains(uri)
  }

  pub(crate) fn dependencies(
    &self,
    source: &lsp::Url,
  ) -> impl Iterator<Item = &ProjectDependency> {
    self.dependencies.get(source).into_iter().flatten()
  }

  #[must_use]
  pub(crate) fn dependency_at(
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

  #[must_use]
  pub(crate) fn new(
    root: lsp::Url,
    dependencies: HashMap<lsp::Url, Vec<ProjectDependency>>,
    targets: HashSet<lsp::Url>,
  ) -> Self {
    Self {
      import_scope: ImportScope::new(&root, &dependencies),
      dependencies,
      root,
      targets,
    }
  }
}
