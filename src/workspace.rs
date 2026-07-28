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
}
