use super::*;

#[derive(Debug)]
pub(crate) struct Project {
  candidates: Vec<lsp::Url>,
  pub(crate) dependencies: HashMap<lsp::Url, Vec<ProjectDependency>>,
  pub(crate) import_scope: ImportScope,
  pub(crate) root: lsp::Url,
  pub(crate) targets: HashSet<lsp::Url>,
}

impl Project {
  #[must_use]
  pub(crate) fn contains(&self, uri: &lsp::Url) -> bool {
    self.root == *uri
      || self.targets.contains(uri)
      || self
        .candidates
        .iter()
        .any(|candidate| candidate.as_str().eq_ignore_ascii_case(uri.as_str()))
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
  pub(crate) fn loads(&self, uri: &lsp::Url) -> bool {
    self.dependencies.contains_key(uri)
  }

  #[must_use]
  pub(crate) fn new(
    root: lsp::Url,
    dependencies: HashMap<lsp::Url, Vec<ProjectDependency>>,
  ) -> Self {
    let targets = dependencies
      .values()
      .flatten()
      .filter_map(|dependency| match &dependency.target {
        ProjectDependencyTarget::Resolved(target) => Some(target.clone()),
        _ => None,
      })
      .collect();

    let candidates = dependencies
      .values()
      .flatten()
      .flat_map(|dependency| match &dependency.target {
        ProjectDependencyTarget::Candidates(candidates) => candidates.clone(),
        _ => Vec::new(),
      })
      .collect();

    Self {
      candidates,
      import_scope: ImportScope::new(&root, &dependencies),
      dependencies,
      root,
      targets,
    }
  }

  #[must_use]
  pub(crate) fn target<'a>(
    &self,
    dependency: &'a ProjectDependency,
  ) -> Option<&'a lsp::Url> {
    match &dependency.target {
      ProjectDependencyTarget::Resolved(target) if self.loads(target) => {
        Some(target)
      }
      _ => None,
    }
  }
}
