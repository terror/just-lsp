use super::*;

#[derive(Debug)]
pub struct ImportScope {
  documents: Vec<ImportScopeDocument>,
}

impl ImportScope {
  #[must_use]
  pub fn documents(&self) -> &[ImportScopeDocument] {
    &self.documents
  }

  pub(super) fn new(uri: lsp::Url) -> Self {
    Self {
      documents: vec![ImportScopeDocument {
        load_depth: 0,
        traversal_order: 0,
        uri,
      }],
    }
  }
}

impl From<&Project> for ImportScope {
  fn from(project: &Project) -> Self {
    let mut depths = HashMap::new();

    let mut stack = vec![(0, project.root.clone())];

    while let Some((depth, source)) = stack.pop() {
      if depths.contains_key(&source) {
        continue;
      }

      depths.insert(source.clone(), depth);

      for dependency in project.dependencies(&source) {
        if let ProjectDependencyTarget::Resolved(target) = &dependency.target {
          stack.push((depth + 1, target.clone()));
        }
      }
    }

    let mut documents = Vec::new();

    let mut seen = HashSet::from([project.root.clone()]);
    let mut stack = vec![project.root.clone()];

    while let Some(source) = stack.pop() {
      documents.push(ImportScopeDocument {
        load_depth: depths[&source],
        traversal_order: documents.len(),
        uri: source.clone(),
      });

      for dependency in project.dependencies(&source) {
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
}
