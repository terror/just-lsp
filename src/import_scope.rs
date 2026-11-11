use super::*;

#[derive(Debug)]
pub struct ImportScope {
  documents: Vec<ImportScopeDocument>,
}

impl ImportScope {
  pub(super) fn contains(&self, uri: &lsp::Url) -> bool {
    self.documents.iter().any(|document| document.uri == *uri)
  }

  #[must_use]
  pub fn documents(&self) -> &[ImportScopeDocument] {
    &self.documents
  }

  pub(super) fn for_root(project: &Project, root: &lsp::Url) -> Self {
    let mut depths = HashMap::new();

    let mut stack = vec![(0, root.clone())];

    while let Some((depth, source)) = stack.pop() {
      if depths.contains_key(&source) {
        continue;
      }

      depths.insert(source.clone(), depth);

      for dependency in project.dependencies(&source).filter(|dependency| {
        matches!(dependency.kind, ProjectDependencyKind::Import)
      }) {
        if let ProjectDependencyTarget::Resolved(target) = &dependency.target {
          stack.push((depth + 1, target.clone()));
        }
      }
    }

    let mut documents = Vec::new();

    let mut seen = HashSet::from([root.clone()]);
    let mut stack = vec![root.clone()];

    while let Some(source) = stack.pop() {
      documents.push(ImportScopeDocument {
        load_depth: depths[&source],
        uri: source.clone(),
      });

      for dependency in project.dependencies(&source).filter(|dependency| {
        matches!(dependency.kind, ProjectDependencyKind::Import)
      }) {
        let ProjectDependencyTarget::Resolved(target) = &dependency.target
        else {
          continue;
        };

        if seen.insert(target.clone()) {
          stack.push(target.clone());
        }
      }
    }

    Self { documents }
  }

  pub(super) fn new(uri: lsp::Url) -> Self {
    Self {
      documents: vec![ImportScopeDocument { load_depth: 0, uri }],
    }
  }
}

impl From<&Project> for ImportScope {
  fn from(project: &Project) -> Self {
    Self::for_root(project, &project.root)
  }
}
