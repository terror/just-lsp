use super::*;

#[derive(Debug)]
pub(crate) struct Project {
  candidates: Vec<lsp::Url>,
  pub(crate) dependencies: HashMap<lsp::Url, Vec<ProjectDependency>>,
  pub(crate) modules: BTreeMap<lsp::Url, Module>,
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

  #[cfg(test)]
  pub(crate) fn module(&self) -> &Module {
    &self.modules[&self.root]
  }

  pub(crate) fn modules_containing(&self, uri: &lsp::Url) -> Vec<&Module> {
    let mut modules = self
      .modules
      .iter()
      .filter(|(_, module)| module.import_scope.contains(uri))
      .collect::<Vec<_>>();

    modules.sort_by_key(|(root, _)| *root != &self.root);

    modules.into_iter().map(|(_, module)| module).collect()
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

    let mut modules = BTreeMap::new();

    let mut stack = vec![root.clone()];

    while let Some(uri) = stack.pop() {
      if modules.contains_key(&uri) {
        continue;
      }

      let module = Module::new(&uri, &dependencies);

      stack.extend(module.children.values().cloned());

      modules.insert(uri, module);
    }

    Self {
      candidates,
      dependencies,
      modules,
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
