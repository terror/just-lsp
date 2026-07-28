use super::*;

#[derive(Debug, Default)]
pub struct Workspace {
  pub documents: DocumentStore,
  pub projects: HashMap<lsp::Url, Project>,
}

impl Workspace {
  /// # Errors
  ///
  /// Returns an [`Error`] if the project root cannot be loaded.
  pub fn load_project(&mut self, root: lsp::Url) -> Result {
    let project = ProjectLoader::load(&mut self.documents, &root)?;

    self.projects.insert(root, project);

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
