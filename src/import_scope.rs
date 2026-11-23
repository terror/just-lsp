use super::*;

#[derive(Debug)]
pub(crate) struct ImportScope {
  documents: Vec<ScopeDocument<lsp::Url>>,
}

impl ImportScope {
  pub(super) fn contains(&self, uri: &lsp::Url) -> bool {
    self
      .documents
      .iter()
      .any(|document| document.document == *uri)
  }

  #[must_use]
  pub(crate) fn documents(&self) -> &[ScopeDocument<lsp::Url>] {
    &self.documents
  }

  pub(super) fn new(
    root: &lsp::Url,
    dependencies: &HashMap<lsp::Url, Vec<ProjectDependency>>,
  ) -> Self {
    let targets = |source: &lsp::Url| {
      dependencies
        .get(source)
        .into_iter()
        .flatten()
        .filter_map(|dependency| match &dependency.target {
          ProjectDependencyTarget::Resolved(target) => Some(target.clone()),
          _ => None,
        })
        .collect::<Vec<_>>()
    };

    let mut depths = HashMap::new();

    let mut stack = vec![(0, root.clone())];

    while let Some((depth, source)) = stack.pop() {
      if depths.contains_key(&source) {
        continue;
      }

      for target in targets(&source) {
        stack.push((depth + 1, target));
      }

      depths.insert(source, depth);
    }

    let mut documents = Vec::new();

    let mut seen = HashSet::from([root.clone()]);
    let mut stack = vec![root.clone()];

    while let Some(source) = stack.pop() {
      for target in targets(&source) {
        if seen.insert(target.clone()) {
          stack.push(target);
        }
      }

      documents.push(ScopeDocument {
        load_depth: depths[&source],
        document: source,
      });
    }

    Self { documents }
  }
}
