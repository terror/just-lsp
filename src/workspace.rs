use super::*;

#[derive(Debug, Default)]
pub struct Workspace {
  documents: DocumentStore,
  projects: HashMap<lsp::Url, Project>,
}

impl Workspace {
  #[must_use]
  fn affected_roots(&self, uri: &lsp::Url) -> HashSet<lsp::Url> {
    self
      .projects
      .iter()
      .filter(|(_, project)| project.contains(uri))
      .map(|(root, _)| root.clone())
      .collect()
  }

  fn analyze_document(
    &self,
    document: &Document,
    config: Option<&Config>,
    projects: &[&Project],
  ) -> Vec<Diagnostic> {
    let analyses = projects
      .iter()
      .filter(|project| project.import_scope.contains(&document.uri))
      .map(|project| {
        Analyzer {
          config,
          view: ProjectView::new(
            document,
            &project.import_scope,
            &self.documents,
          ),
        }
        .analyze()
      })
      .collect::<Vec<_>>();

    let mut quickfixes = analyses
      .first()
      .into_iter()
      .flatten()
      .flat_map(|diagnostic| diagnostic.quickfixes.iter().cloned())
      .collect::<Vec<_>>();

    for diagnostics in analyses.iter().skip(1) {
      quickfixes.retain(|quickfix| {
        diagnostics
          .iter()
          .any(|diagnostic| diagnostic.quickfixes.contains(quickfix))
      });
    }

    let mut diagnostics = Vec::<Diagnostic>::new();

    for mut diagnostic in analyses.into_iter().flatten() {
      diagnostic
        .quickfixes
        .retain(|quickfix| quickfixes.contains(quickfix));

      if let Some(previous) = diagnostics.iter_mut().find(|previous| {
        previous.id == diagnostic.id
          && previous.message == diagnostic.message
          && previous.range == diagnostic.range
          && previous.severity == diagnostic.severity
      }) {
        for quickfix in diagnostic.quickfixes {
          if !previous.quickfixes.contains(&quickfix) {
            previous.quickfixes.push(quickfix);
          }
        }
      } else {
        diagnostics.push(diagnostic);
      }
    }

    diagnostics.sort_by(|left, right| {
      left
        .range
        .start
        .cmp(&right.range.start)
        .then_with(|| left.message.cmp(&right.message))
    });

    diagnostics
  }

  /// Applies changes to an open document and rebuilds affected projects.
  ///
  /// Returns `false` if the document is not open.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] if the changed document cannot be parsed or an
  /// affected project root cannot be loaded.
  pub fn change(
    &mut self,
    params: lsp::DidChangeTextDocumentParams,
  ) -> Result<bool> {
    let uri = &params.text_document.uri;

    if !self.documents.is_open(uri) {
      return Ok(false);
    }

    let roots = self.affected_roots(uri);

    self.documents.change(params)?;

    self.load_projects(roots)?;

    Ok(true)
  }

  /// Closes a document and rebuilds affected projects.
  ///
  /// Restores the document from disk, or removes it if it cannot be loaded.
  /// Returns `false` if the document is not open.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] if an affected project root cannot be loaded.
  pub fn close(
    &mut self,
    params: &lsp::DidCloseTextDocumentParams,
  ) -> Result<bool> {
    let uri = &params.text_document.uri;

    let mut roots = self.affected_roots(uri);

    if !self.documents.close(params) {
      return Ok(false);
    }

    if self.documents.get(uri).is_none() {
      self.projects.remove(uri);
      roots.remove(uri);
    }

    self.load_projects(roots)?;

    Ok(true)
  }

  #[must_use]
  pub fn diagnostics(
    &self,
    config: Option<&Config>,
  ) -> BTreeMap<lsp::Url, Vec<Diagnostic>> {
    let projects = self.root_projects();

    projects
      .iter()
      .flat_map(|project| project.import_scope.documents())
      .map(|document| &document.uri)
      .collect::<BTreeSet<_>>()
      .into_iter()
      .filter_map(|uri| self.documents.get(uri))
      .map(|document| {
        (
          document.uri.clone(),
          self.analyze_document(document, config, &projects),
        )
      })
      .collect()
  }

  #[must_use]
  pub fn document(&self, uri: &lsp::Url) -> Option<&Document> {
    self.documents.get(uri)
  }

  #[must_use]
  pub fn document_diagnostics(
    &self,
    uri: &lsp::Url,
    config: Option<&Config>,
  ) -> Vec<Diagnostic> {
    self.documents.get(uri).map_or_else(Vec::new, |document| {
      self.analyze_document(document, config, &self.root_projects())
    })
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
  fn load_projects(
    &mut self,
    roots: impl IntoIterator<Item = lsp::Url>,
  ) -> Result {
    for root in roots {
      self.load_project(root)?;
    }

    self.projects.retain(|_, project| {
      project
        .import_scope
        .documents()
        .iter()
        .any(|document| self.documents.is_open(&document.uri))
    });

    self.documents.retain_closed(|uri| {
      self
        .projects
        .values()
        .any(|project| project.import_scope.contains(uri))
    });

    Ok(())
  }

  /// Opens a document and rebuilds its project and affected projects.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] if the opened document cannot be parsed or a
  /// project root cannot be loaded.
  pub fn open(&mut self, params: lsp::DidOpenTextDocumentParams) -> Result {
    let mut roots = self.affected_roots(&params.text_document.uri);

    roots.insert(params.text_document.uri.clone());

    self.documents.open(params)?;

    self.load_projects(roots)
  }

  #[must_use]
  pub fn open_document(&self, uri: &lsp::Url) -> Option<&Document> {
    self.documents.get_open(uri)
  }

  #[must_use]
  pub fn project(&self, uri: &lsp::Url) -> Option<&Project> {
    self.projects.get(uri)
  }

  #[must_use]
  pub fn project_view(&self, uri: &lsp::Url) -> Option<ProjectView<'_>> {
    let document = self.documents.get_open(uri)?;

    let project = self
      .root_projects()
      .into_iter()
      .find(|project| project.import_scope.contains(uri));

    Some(project.map_or_else(
      || ProjectView::from(document),
      |project| {
        ProjectView::new(document, &project.import_scope, &self.documents)
      },
    ))
  }

  fn root_projects(&self) -> Vec<&Project> {
    let mut projects = self
      .projects
      .values()
      .filter(|project| {
        !self.projects.values().any(|other| {
          other.root != project.root
            && other.import_scope.contains(&project.root)
            && (!project.import_scope.contains(&other.root)
              || other.root < project.root)
        })
      })
      .collect::<Vec<_>>();

    projects.sort_by_key(|project| &project.root);

    projects
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn change_ignores_closed_document() {
    let tempdir = tempfile::tempdir().unwrap();

    let root = tempdir.path().join("justfile");

    fs::write(&root, "foo:").unwrap();

    let root = lsp::Url::from_file_path(root).unwrap();

    let mut workspace = Workspace::default();

    workspace.load_project(root.clone()).unwrap();

    assert!(
      !workspace
        .change(lsp::DidChangeTextDocumentParams {
          text_document: lsp::VersionedTextDocumentIdentifier {
            uri: root.clone(),
            version: 1,
          },
          content_changes: vec![lsp::TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "bar:".into(),
          }],
        })
        .unwrap()
    );

    assert_eq!(
      workspace.document(&root).unwrap().content.to_string(),
      "foo:",
    );

    assert!(workspace.open_document(&root).is_none());

    assert_eq!(
      workspace.projects.keys().collect::<HashSet<_>>(),
      HashSet::from([&root]),
    );
  }

  #[test]
  fn change_ignores_unknown_document() {
    let uri = lsp::Url::parse("file:///foo.just").unwrap();

    let mut workspace = Workspace::default();

    assert!(
      !workspace
        .change(lsp::DidChangeTextDocumentParams {
          text_document: lsp::VersionedTextDocumentIdentifier {
            uri: uri.clone(),
            version: 1,
          },
          content_changes: vec![lsp::TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "bar:".into(),
          }],
        })
        .unwrap()
    );

    assert!(workspace.document(&uri).is_none());
    assert!(workspace.projects.is_empty());
  }

  #[test]
  fn close_ignores_closed_document() {
    let tempdir = tempfile::tempdir().unwrap();

    let root = tempdir.path().join("justfile");
    fs::write(&root, "foo:").unwrap();

    let root = lsp::Url::from_file_path(root).unwrap();

    let mut workspace = Workspace::default();

    workspace.load_project(root.clone()).unwrap();

    assert!(
      !workspace
        .close(&lsp::DidCloseTextDocumentParams {
          text_document: lsp::TextDocumentIdentifier { uri: root.clone() },
        })
        .unwrap()
    );

    assert_eq!(
      workspace.document(&root).unwrap().content.to_string(),
      "foo:",
    );

    assert!(workspace.open_document(&root).is_none());

    assert_eq!(
      workspace.projects.keys().collect::<HashSet<_>>(),
      HashSet::from([&root]),
    );
  }

  #[test]
  fn close_ignores_unknown_document() {
    let uri = lsp::Url::parse("file:///foo.just").unwrap();

    let mut workspace = Workspace::default();

    assert!(
      !workspace
        .close(&lsp::DidCloseTextDocumentParams {
          text_document: lsp::TextDocumentIdentifier { uri: uri.clone() },
        })
        .unwrap()
    );

    assert!(workspace.document(&uri).is_none());
    assert!(workspace.projects.is_empty());
  }

  #[test]
  fn unloaded_projects_reload_imports() {
    let tempdir = tempfile::tempdir().unwrap();

    let root = tempdir.path().join("justfile");
    let imported = tempdir.path().join("foo.just");

    fs::write(&root, "import 'foo.just'").unwrap();
    fs::write(&imported, "foo:").unwrap();

    let root = lsp::Url::from_file_path(root).unwrap();

    let mut workspace = Workspace::default();

    let params = lsp::DidOpenTextDocumentParams {
      text_document: lsp::TextDocumentItem {
        uri: root.clone(),
        language_id: "just".into(),
        version: 1,
        text: "import 'foo.just'".into(),
      },
    };

    workspace.open(params.clone()).unwrap();

    assert!(
      workspace
        .close(&lsp::DidCloseTextDocumentParams {
          text_document: lsp::TextDocumentIdentifier { uri: root.clone() },
        })
        .unwrap()
    );

    assert!(workspace.projects.is_empty());
    assert!(workspace.document(&root).is_none());

    fs::write(&imported, "bar:").unwrap();

    workspace.open(params).unwrap();

    let imported = lsp::Url::from_file_path(imported).unwrap();

    assert_eq!(
      workspace.document(&imported).unwrap().content.to_string(),
      "bar:",
    );
  }
}
