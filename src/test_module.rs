use super::*;

pub(crate) struct TestModule {
  directory: PathBuf,
  documents: DocumentStore,
  project: Project,
}

impl TestModule {
  pub(crate) fn new(files: &[(&str, &str)]) -> Self {
    let directory = if cfg!(windows) {
      PathBuf::from("C:\\")
    } else {
      PathBuf::from("/")
    };

    let mut documents =
      DocumentStore::new(Arc::new(MemoryFilesystem::new(files.iter().map(
        |(path, content)| (directory.join(path), (*content).to_owned()),
      ))));

    let root = lsp::Url::from_file_path(directory.join("justfile")).unwrap();

    let project = ProjectLoader::load(&mut documents, &root).unwrap();

    Self {
      directory,
      documents,
      project,
    }
  }

  pub(crate) fn uri(&self, path: &str) -> lsp::Url {
    lsp::Url::from_file_path(self.directory.join(path)).unwrap()
  }

  pub(crate) fn view(&self) -> ModuleView<'_> {
    ModuleView::new(
      self.documents.get(&self.project.root).unwrap(),
      ModuleContext {
        documents: &self.documents,
        module: self.project.module(),
        project: &self.project,
      },
    )
  }
}
