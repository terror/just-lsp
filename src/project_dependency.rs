use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProjectDependency {
  pub(crate) kind: ProjectDependencyKind,
  pub(crate) location: lsp::Range,
  pub(crate) optional: bool,
  pub(crate) target: ProjectDependencyTarget,
}

impl ProjectDependency {
  pub(crate) fn import(source: &lsp::Url, import: &Import) -> Self {
    let target = if import.is_dynamic() {
      ProjectDependencyTarget::Dynamic
    } else {
      match import.resolve(source) {
        Ok(Some(path)) => Self::target(&path),
        Ok(None) | Err(Error::EmptyImportPath) => {
          ProjectDependencyTarget::Missing
        }
        Err(_) => ProjectDependencyTarget::Dynamic,
      }
    };

    Self {
      kind: ProjectDependencyKind::Import,
      location: import.path.range,
      optional: import.optional,
      target,
    }
  }

  pub(crate) fn missing_module(&self) -> Option<&str> {
    match (&self.kind, &self.target) {
      (
        ProjectDependencyKind::Module { name },
        ProjectDependencyTarget::Candidates(_),
      ) if !self.optional => Some(name),
      _ => None,
    }
  }

  pub(crate) fn module(
    source: &lsp::Url,
    module: &ModuleDeclaration,
    filesystem: &dyn Filesystem,
  ) -> Self {
    let target = if module.is_dynamic() {
      ProjectDependencyTarget::Dynamic
    } else {
      module.resolve(source, filesystem).map_or_else(
        || {
          ProjectDependencyTarget::Candidates(
            module
              .candidates(source)
              .iter()
              .filter_map(|(path, _)| lsp::Url::from_path(&path.clean()))
              .collect(),
          )
        },
        |path| Self::target(&path),
      )
    };

    Self {
      kind: ProjectDependencyKind::Module {
        name: module.name.value.clone(),
      },
      location: module.location(),
      optional: module.optional,
      target,
    }
  }

  fn target(path: &Path) -> ProjectDependencyTarget {
    lsp::Url::from_path(&path.clean()).map_or(
      ProjectDependencyTarget::Missing,
      ProjectDependencyTarget::Path,
    )
  }
}
