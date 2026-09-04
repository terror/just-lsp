use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectDependency {
  pub kind: ProjectDependencyKind,
  pub location: lsp::Range,
  pub target: ProjectDependencyTarget,
}

impl ProjectDependency {
  #[must_use]
  pub fn is_enabled(&self) -> bool {
    match &self.kind {
      ProjectDependencyKind::Import { attributes, .. } => {
        platform::attributes_enabled(attributes)
      }
    }
  }
}
