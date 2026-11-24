use super::*;

#[derive(Debug)]
pub(crate) struct DocumentStore {
  documents: HashMap<lsp::Url, DocumentEntry>,
  filesystem: Arc<dyn Filesystem>,
}

impl DocumentStore {
  /// # Errors
  ///
  /// Returns an [`Error`] if the changed document cannot be parsed.
  pub(crate) fn change(
    &mut self,
    params: lsp::DidChangeTextDocumentParams,
  ) -> Result {
    let Some(entry) = self.documents.get_mut(&params.text_document.uri) else {
      return Ok(());
    };

    if !entry.open {
      return Ok(());
    }

    entry.document.apply_change(params)?;

    entry.imports = OnceLock::new();
    entry.modules = OnceLock::new();

    Ok(())
  }

  pub(crate) fn close(
    &mut self,
    params: &lsp::DidCloseTextDocumentParams,
  ) -> bool {
    let uri = &params.text_document.uri;

    if !self.is_open(uri) {
      return false;
    }

    match Self::read(&*self.filesystem, uri) {
      Ok(document) => {
        self
          .documents
          .insert(uri.clone(), DocumentEntry::new(document, false));
      }
      Err(_) => {
        self.documents.remove(uri);
      }
    }

    true
  }

  pub(crate) fn dependencies(
    &self,
    uri: &lsp::Url,
  ) -> Option<Vec<ProjectDependency>> {
    let entry = self.documents.get(uri)?;

    let imports = entry.imports.get_or_init(|| {
      entry
        .document
        .imports()
        .into_iter()
        .filter(Import::is_enabled)
        .map(|import| ProjectDependency::import(uri, &import))
        .collect()
    });

    let modules = entry.modules.get_or_init(|| {
      entry
        .document
        .modules()
        .into_iter()
        .filter(ModuleDeclaration::is_enabled)
        .collect()
    });

    Some(
      imports
        .iter()
        .cloned()
        .chain(modules.iter().map(|module| {
          ProjectDependency::module(uri, module, &*self.filesystem)
        }))
        .collect(),
    )
  }

  pub(crate) fn filesystem(&self) -> &dyn Filesystem {
    &*self.filesystem
  }

  #[must_use]
  pub(crate) fn get(&self, uri: &lsp::Url) -> Option<&Document> {
    self.documents.get(uri).map(|entry| &entry.document)
  }

  #[must_use]
  pub(crate) fn get_open(&self, uri: &lsp::Url) -> Option<&Document> {
    self
      .documents
      .get(uri)
      .filter(|entry| entry.open)
      .map(|entry| &entry.document)
  }

  #[must_use]
  pub(crate) fn is_open(&self, uri: &lsp::Url) -> bool {
    self.documents.get(uri).is_some_and(|entry| entry.open)
  }

  /// # Errors
  ///
  /// Returns an [`Error`] if the URI is not a file URI, the file cannot be
  /// read, or the document cannot be parsed.
  pub(crate) fn load(&mut self, uri: &lsp::Url) -> Result<&Document> {
    match self.documents.entry(uri.clone()) {
      Entry::Occupied(entry) => Ok(&entry.into_mut().document),
      Entry::Vacant(entry) => Ok(
        &entry
          .insert(DocumentEntry::new(
            Self::read(&*self.filesystem, uri)?,
            false,
          ))
          .document,
      ),
    }
  }

  pub(crate) fn new(filesystem: Arc<dyn Filesystem>) -> Self {
    Self {
      documents: HashMap::new(),
      filesystem,
    }
  }

  /// # Errors
  ///
  /// Returns an [`Error`] if the opened document cannot be parsed.
  pub(crate) fn open(
    &mut self,
    params: lsp::DidOpenTextDocumentParams,
  ) -> Result {
    let document = Document::try_from(params)?;

    self
      .documents
      .insert(document.uri.clone(), DocumentEntry::new(document, true));

    Ok(())
  }

  fn read(filesystem: &dyn Filesystem, uri: &lsp::Url) -> Result<Document> {
    let path = uri.file_path()?;

    Document::new(&filesystem.read_to_string(&path)?, uri.clone())
  }

  pub(super) fn retain_closed(
    &mut self,
    mut retain: impl FnMut(&lsp::Url) -> bool,
  ) {
    self
      .documents
      .retain(|uri, entry| entry.open || retain(uri));
  }
}

impl Default for DocumentStore {
  fn default() -> Self {
    Self::new(Arc::new(OsFilesystem))
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  fn change(
    uri: lsp::Url,
    version: i32,
    text: &str,
  ) -> lsp::DidChangeTextDocumentParams {
    lsp::DidChangeTextDocumentParams {
      text_document: lsp::VersionedTextDocumentIdentifier { uri, version },
      content_changes: vec![lsp::TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: text.into(),
      }],
    }
  }

  fn close(uri: lsp::Url) -> lsp::DidCloseTextDocumentParams {
    lsp::DidCloseTextDocumentParams {
      text_document: lsp::TextDocumentIdentifier { uri },
    }
  }

  fn entry(name: &str, open: bool) -> (lsp::Url, DocumentEntry) {
    let uri = lsp::Url::parse(&format!("file:///{name}")).unwrap();

    (
      uri.clone(),
      DocumentEntry::new(Document::new("", uri).unwrap(), open),
    )
  }

  fn open(
    uri: lsp::Url,
    version: i32,
    text: &str,
  ) -> lsp::DidOpenTextDocumentParams {
    lsp::DidOpenTextDocumentParams {
      text_document: lsp::TextDocumentItem {
        uri,
        language_id: "just".into(),
        version,
        text: text.into(),
      },
    }
  }

  fn uri(path: &Path) -> lsp::Url {
    lsp::Url::from_file_path(path).unwrap()
  }

  #[test]
  fn change_clears_dependencies() {
    let tempdir = tempfile::tempdir().unwrap();

    let foo = uri(&tempdir.path().join("foo.just"));

    let mut store = DocumentStore::default();

    store
      .open(open(foo.clone(), 1, "import 'bar.just'"))
      .unwrap();

    assert_eq!(
      store.dependencies(&foo).unwrap(),
      [ProjectDependency {
        kind: ProjectDependencyKind::Import,
        location: lsp::Range::at(0, 7, 0, 17),
        optional: false,
        target: ProjectDependencyTarget::Path(uri(
          &tempdir.path().join("bar.just")
        )),
      }],
    );

    store.change(change(foo.clone(), 2, "")).unwrap();

    assert_eq!(store.dependencies(&foo).unwrap(), []);
  }

  #[test]
  fn change_updates_open_document() {
    let uri = lsp::Url::parse("file:///foo.just").unwrap();

    let mut store = DocumentStore::default();

    store.open(open(uri.clone(), 1, "foo:")).unwrap();
    store.change(change(uri.clone(), 2, "bar:")).unwrap();

    let document = store.get_open(&uri).unwrap();

    assert_eq!(document.content.to_string(), "bar:");
    assert_eq!(document.version, 2);
  }

  #[test]
  fn close_reloads_disk_document() {
    let tempdir = tempfile::tempdir().unwrap();

    let path = tempdir.path().join("foo.just");
    let uri = uri(&path);

    fs::write(path, "foo:").unwrap();

    let mut store = DocumentStore::default();

    store.open(open(uri.clone(), 1, "bar:")).unwrap();

    assert!(store.close(&close(uri.clone())));
    assert!(!store.is_open(&uri));
    assert_eq!(store.get(&uri).unwrap().content.to_string(), "foo:");
    assert!(store.get_open(&uri).is_none());
  }

  #[test]
  fn load_caches_disk_document() {
    let tempdir = tempfile::tempdir().unwrap();

    let path = tempdir.path().join("foo.just");
    let uri = uri(&path);

    fs::write(&path, "foo:").unwrap();

    let mut store = DocumentStore::default();

    assert_eq!(store.load(&uri).unwrap().content.to_string(), "foo:");

    fs::write(path, "bar:").unwrap();

    assert_eq!(store.load(&uri).unwrap().content.to_string(), "foo:");

    assert!(!store.is_open(&uri));
  }

  #[test]
  fn load_prefers_open_document() {
    let tempdir = tempfile::tempdir().unwrap();

    let path = tempdir.path().join("foo.just");
    let uri = uri(&path);

    fs::write(path, "foo:").unwrap();

    let mut store = DocumentStore::default();

    store.load(&uri).unwrap();
    store.open(open(uri.clone(), 1, "bar:")).unwrap();

    assert_eq!(store.load(&uri).unwrap().content.to_string(), "bar:");

    assert!(store.is_open(&uri));
  }

  #[test]
  fn module_dependencies_follow_filesystem() {
    let tempdir = tempfile::tempdir().unwrap();

    let foo = uri(&tempdir.path().join("foo.just"));

    let mut store = DocumentStore::default();

    store.open(open(foo.clone(), 1, "mod bar")).unwrap();

    let dependency = |target| ProjectDependency {
      kind: ProjectDependencyKind::Module { name: "bar".into() },
      location: lsp::Range::at(0, 4, 0, 7),
      optional: false,
      target,
    };

    assert_eq!(
      store.dependencies(&foo).unwrap(),
      [dependency(ProjectDependencyTarget::Candidates(
        ["bar.just", "bar/mod.just", "bar/justfile", "bar/.justfile"]
          .iter()
          .map(|path| uri(&tempdir.path().join(path)))
          .collect()
      ))],
    );

    fs::write(tempdir.path().join("bar.just"), "").unwrap();

    assert_eq!(
      store.dependencies(&foo).unwrap(),
      [dependency(ProjectDependencyTarget::Path(uri(
        &tempdir.path().join("bar.just")
      )))],
    );
  }

  #[test]
  fn retain_closed_documents() {
    let mut store = DocumentStore {
      documents: HashMap::from([
        entry("foo", false),
        entry("bar", false),
        entry("baz", true),
      ]),
      ..DocumentStore::default()
    };

    store.retain_closed(|uri| uri.as_str() == "file:///foo");

    assert_eq!(
      store
        .documents
        .iter()
        .map(|(uri, entry)| (uri.as_str(), entry.open))
        .collect::<BTreeMap<_, _>>(),
      BTreeMap::from([("file:///baz", true), ("file:///foo", false)]),
    );
  }
}
