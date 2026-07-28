use super::*;

#[derive(Debug, Default)]
pub struct Workspace {
  pub documents: DocumentStore,
  pub projects: HashMap<lsp::Url, Project>,
}

impl Workspace {
  #[must_use]
  pub fn affected_roots(&self, uri: &lsp::Url) -> HashSet<lsp::Url> {
    self
      .projects
      .iter()
      .filter(|(_, project)| project.contains(uri))
      .map(|(root, _)| root.clone())
      .collect()
  }

  /// # Errors
  ///
  /// Returns an [`Error`] if the project root cannot be loaded.
  pub fn load_project(&mut self, root: lsp::Url) -> Result {
    let project = ProjectLoader::load(&mut self.documents, &root)?;

    self.projects.insert(root, project);

    Ok(())
  }

  /// # Errors
  ///
  /// Returns an [`Error`] if a project root cannot be loaded.
  pub fn load_projects(
    &mut self,
    roots: impl IntoIterator<Item = lsp::Url>,
  ) -> Result {
    for root in roots {
      self.load_project(root)?;
    }

    Ok(())
  }

  #[must_use]
  pub fn project_view(&self, uri: &lsp::Url) -> Option<ProjectView<'_>> {
    let document = self.documents.get_open(uri)?;

    let imported_documents = self
      .projects
      .get(uri)
      .into_iter()
      .flat_map(|project| project.imported_documents(&self.documents));

    Some(ProjectView::new(document, imported_documents))
  }
}
