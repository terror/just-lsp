use super::*;

#[derive(Debug, Default)]
pub struct Workspace {
  config: Config,
  documents: DocumentStore,
  project_configs: HashMap<lsp::Url, Option<Config>>,
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

  fn analyze(
    &self,
    document: &Document,
    projects: &[&Project],
  ) -> Vec<Diagnostic> {
    Diagnostic::merge(
      &Self::contexts(projects, &document.uri)
        .into_iter()
        .map(|(project, module)| {
          let Some(config) = self
            .project_configs
            .get(&project.root)
            .and_then(Option::as_ref)
          else {
            return Vec::new();
          };

          let analyzer = Analyzer {
            config: Some(&self.config.merge(config)),
            view: ModuleView::new(
              document,
              ModuleContext {
                documents: &self.documents,
                module,
                project,
              },
            ),
          };

          analyzer.analyze()
        })
        .collect::<Vec<_>>(),
    )
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

    self.load_projects(roots, false)?;

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

    self.load_projects(roots, false)?;

    Ok(true)
  }

  fn contexts<'a>(
    projects: &[&'a Project],
    uri: &lsp::Url,
  ) -> Vec<(&'a Project, &'a Module)> {
    projects
      .iter()
      .flat_map(|project| {
        project
          .modules_containing(uri)
          .into_iter()
          .map(move |module| (*project, module))
      })
      .collect()
  }

  #[must_use]
  pub fn definition(
    &self,
    uri: &lsp::Url,
    position: lsp::Position,
  ) -> Option<lsp::Location> {
    if let Some(target) = self
      .projects
      .get(uri)
      .and_then(|project| project.target(project.dependency_at(uri, position)?))
    {
      return Some(lsp::Location::new(target.clone(), lsp::Range::default()));
    }

    let view = self.view(uri)?;

    let identifier = view
      .document()
      .node_at_position(position)
      .filter(|node| node.kind() == "identifier")?;

    Resolver::new(view).resolve_identifier_definition(&identifier)
  }

  #[must_use]
  pub fn diagnostics(&self) -> BTreeMap<lsp::Url, Vec<Diagnostic>> {
    let projects = self.root_projects();

    projects
      .iter()
      .filter(|project| {
        self
          .project_configs
          .get(&project.root)
          .is_some_and(Option::is_some)
      })
      .flat_map(|project| project.modules.values())
      .flat_map(|module| module.import_scope.documents())
      .map(|document| &document.document)
      .collect::<BTreeSet<_>>()
      .into_iter()
      .filter_map(|uri| self.documents.get(uri))
      .map(|document| (document.uri.clone(), self.analyze(document, &projects)))
      .collect()
  }

  #[must_use]
  pub fn document(&self, uri: &lsp::Url) -> Option<&Document> {
    self.documents.get(uri)
  }

  #[must_use]
  pub fn document_config(&self, uri: &lsp::Url) -> Option<Config> {
    Self::contexts(&self.root_projects(), uri)
      .first()
      .map_or_else(
        || Some(self.config.clone()),
        |(project, _)| {
          self
            .project_configs
            .get(&project.root)
            .and_then(Option::as_ref)
            .map(|config| self.config.merge(config))
        },
      )
  }

  #[must_use]
  pub fn document_diagnostics(&self, uri: &lsp::Url) -> Vec<Diagnostic> {
    self.documents.get(uri).map_or_else(Vec::new, |document| {
      self.analyze(document, &self.root_projects())
    })
  }

  #[must_use]
  pub fn links(&self, uri: &lsp::Url) -> Option<Vec<lsp::DocumentLink>> {
    self.documents.get_open(uri)?;

    Some(
      self
        .projects
        .get(uri)
        .into_iter()
        .flat_map(|project| {
          project.dependencies(uri).filter_map(|dependency| {
            let target = project.target(dependency)?;

            Some(lsp::DocumentLink {
              range: dependency.location,
              target: Some(target.clone()),
              tooltip: target
                .file_path()
                .ok()
                .map(|path| path.display().to_string()),
              data: None,
            })
          })
        })
        .collect(),
    )
  }

  /// # Errors
  ///
  /// Returns an [`Error`] if the project root cannot be loaded.
  pub fn load_project(&mut self, root: lsp::Url) -> Result {
    self.load_projects([root], true)
  }

  /// # Errors
  ///
  /// Returns an [`Error`] if a project root cannot be loaded.
  fn load_projects(
    &mut self,
    roots: impl IntoIterator<Item = lsp::Url>,
    retain_closed: bool,
  ) -> Result {
    let roots = roots.into_iter().collect::<HashSet<_>>();

    let projects = roots
      .iter()
      .map(|root| {
        let project = ProjectLoader::load(&mut self.documents, root)?;

        Ok((root.clone(), project))
      })
      .collect::<Result<HashMap<_, _>>>()?;

    self.projects.extend(projects);

    if !retain_closed {
      self.projects.retain(|_, project| {
        project.modules.values().any(|module| {
          module
            .import_scope
            .documents()
            .iter()
            .any(|document| self.documents.is_open(&document.document))
        })
      });

      self.documents.retain_closed(|uri| {
        self.projects.values().any(|project| project.loads(uri))
      });
    }

    let active = self
      .root_projects()
      .into_iter()
      .map(|project| project.root.clone())
      .collect::<BTreeSet<_>>();

    self.project_configs.retain(|root, _| active.contains(root));

    let errors = active
      .into_iter()
      .filter_map(|root| {
        if !roots.contains(&root) && self.project_configs.contains_key(&root) {
          return None;
        }

        let config = if let Ok(path) = root.file_path() {
          Config::find(&path)
        } else {
          Ok(Config::default())
        };

        let (config, error) = match config {
          Ok(config) => (Some(config), None),
          Err(error) => (None, Some(error)),
        };

        self.project_configs.insert(root, config);

        error
      })
      .collect::<Vec<_>>();

    errors.into_iter().next().map_or(Ok(()), Err)
  }

  /// Opens a document and rebuilds its project and affected projects.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] if the opened document cannot be parsed or a
  /// project root cannot be loaded.
  pub fn open(&mut self, params: lsp::DidOpenTextDocumentParams) -> Result {
    let uri = params.text_document.uri.clone();

    let mut roots = self.affected_roots(&uri);

    self.documents.open(params)?;

    if roots.is_empty() {
      roots.extend(self.parent_root(&uri));
    }

    roots.insert(uri);

    self.load_projects(roots, false)
  }

  #[must_use]
  pub fn open_document(&self, uri: &lsp::Url) -> Option<&Document> {
    self.documents.get_open(uri)
  }

  fn parent_root(&mut self, uri: &lsp::Url) -> Option<lsp::Url> {
    let path = uri.file_path().ok()?;

    for directory in path.ancestors().skip(1) {
      for justfile in self.documents.filesystem().justfiles(directory) {
        if justfile == path {
          continue;
        }

        let Some(root) = lsp::Url::from_path(&justfile) else {
          continue;
        };

        if ProjectLoader::load(&mut self.documents, &root)
          .is_ok_and(|project| project.loads(uri))
        {
          return Some(root);
        }
      }
    }

    None
  }

  /// # Errors
  ///
  /// Returns an [`Error`] if an affected project root cannot be loaded.
  pub fn reload(&mut self, uris: &[lsp::Url]) -> Result<bool> {
    let configs = uris.iter().any(|uri| {
      uri
        .file_path()
        .is_ok_and(|path| path.ends_with("just-lsp.toml"))
    });

    let roots = uris
      .iter()
      .flat_map(|uri| self.affected_roots(uri))
      .collect::<HashSet<_>>();

    if !configs && roots.is_empty() {
      return Ok(false);
    }

    if configs {
      self.project_configs.clear();
    }

    for uri in uris {
      self.documents.evict(uri);
    }

    let (roots, missing) = roots
      .into_iter()
      .partition::<HashSet<_>, _>(|root| self.documents.load(root).is_ok());

    for root in missing {
      self.projects.remove(&root);
    }

    self.load_projects(roots, false)?;

    Ok(true)
  }

  fn root_projects(&self) -> Vec<&Project> {
    let mut projects = self
      .projects
      .values()
      .filter(|project| {
        !self.projects.values().any(|other| {
          other.root != project.root
            && !other.modules_containing(&project.root).is_empty()
            && (project.modules_containing(&other.root).is_empty()
              || other.root < project.root)
        })
      })
      .collect::<Vec<_>>();

    projects.sort_by_key(|project| &project.root);

    projects
  }

  #[must_use]
  pub fn run_context(&self, uri: &lsp::Url) -> (lsp::Url, Vec<String>) {
    let contexts = Self::contexts(&self.root_projects(), uri);

    contexts
      .iter()
      .find(|(project, module)| {
        project
          .modules
          .get(uri)
          .is_some_and(|root| ptr::eq(root, *module))
      })
      .or(contexts.first())
      .and_then(|(project, module)| {
        Some((project.root.clone(), project.module_path(module)?))
      })
      .unwrap_or_else(|| (uri.clone(), Vec::new()))
  }

  pub fn set_config(&mut self, config: Config) {
    self.config = config;
  }

  #[must_use]
  pub fn view(&self, uri: &lsp::Url) -> Option<ModuleView<'_>> {
    let document = self.documents.get_open(uri)?;

    Some(
      Self::contexts(&self.root_projects(), uri)
        .first()
        .map_or_else(
          || ModuleView::from(document),
          |(project, module)| {
            ModuleView::new(
              document,
              ModuleContext {
                documents: &self.documents,
                module,
                project,
              },
            )
          },
        ),
    )
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
  fn configuration_error_can_be_fixed() {
    let tempdir = tempfile::tempdir().unwrap();

    let path = tempdir.path().join("just-lsp.toml");

    let uri =
      lsp::Url::from_file_path(tempdir.path().join("justfile")).unwrap();

    fs::write(&path, "foo").unwrap();

    let mut workspace = Workspace::default();

    assert_eq!(
      workspace
        .open(lsp::DidOpenTextDocumentParams {
          text_document: lsp::TextDocumentItem::new(
            uri.clone(),
            "just".into(),
            1,
            "foo: bar\n".into(),
          ),
        })
        .unwrap_err()
        .to_string(),
      format!("failed to parse configuration `{}`", path.display()),
    );

    fs::write(&path, "[rules]\nunresolved-dependency = 'off'").unwrap();

    assert!(
      workspace
        .change(lsp::DidChangeTextDocumentParams {
          text_document: lsp::VersionedTextDocumentIdentifier::new(
            uri.clone(),
            2
          ),
          content_changes: vec![lsp::TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "foo: bar\n".into(),
          }],
        })
        .unwrap()
    );

    assert_eq!(workspace.diagnostics(), BTreeMap::from([(uri, Vec::new())]));
  }

  #[test]
  fn configuration_error_in_unrelated_project() {
    let tempdir = tempfile::tempdir().unwrap();

    let uri = |name: &str| {
      lsp::Url::from_file_path(tempdir.path().join(name).join("justfile"))
        .unwrap()
    };

    let mut workspace = Workspace::default();

    for (name, level) in [("foo", "off"), ("bar", "warning")] {
      let path = tempdir.path().join(name);

      fs::create_dir(&path).unwrap();
      fs::write(
        path.join("just-lsp.toml"),
        format!("[rules]\nunresolved-dependency = '{level}'\n"),
      )
      .unwrap();

      workspace
        .open(lsp::DidOpenTextDocumentParams {
          text_document: lsp::TextDocumentItem::new(
            uri(name),
            "just".into(),
            1,
            "foo: bar\n".into(),
          ),
        })
        .unwrap();
    }

    fs::write(tempdir.path().join("foo/just-lsp.toml"), "foo").unwrap();

    assert!(
      workspace
        .change(lsp::DidChangeTextDocumentParams {
          text_document: lsp::VersionedTextDocumentIdentifier::new(
            uri("bar"),
            2
          ),
          content_changes: vec![lsp::TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "foo:\n".into(),
          }],
        })
        .unwrap()
    );

    assert_eq!(
      workspace.diagnostics(),
      BTreeMap::from([(uri("foo"), Vec::new()), (uri("bar"), Vec::new())]),
    );
  }

  #[test]
  fn document_config_without_file_path() {
    let uri = lsp::Url::parse("untitled:foo").unwrap();

    let config = Config {
      formatting: FormattingConfig {
        indentation: Some("\t".into()),
      },
      ..Default::default()
    };

    let mut workspace = Workspace::default();

    workspace.set_config(config.clone());

    assert_eq!(workspace.document_config(&uri), Some(config.clone()));

    workspace
      .open(lsp::DidOpenTextDocumentParams {
        text_document: lsp::TextDocumentItem::new(
          uri.clone(),
          "just".into(),
          1,
          "foo:\n".into(),
        ),
      })
      .unwrap();

    assert_eq!(workspace.document_config(&uri), Some(config));
  }

  #[test]
  fn module_links_refresh_when_module_opens() {
    #[track_caller]
    fn case(justfile: &str, path: &str, range: lsp::Range) {
      let tempdir = tempfile::tempdir().unwrap();

      let uri = |path: &str| {
        lsp::Url::from_file_path(tempdir.path().join(path)).unwrap()
      };

      let open = |uri: lsp::Url, text: &str| lsp::DidOpenTextDocumentParams {
        text_document: lsp::TextDocumentItem::new(
          uri,
          "just".into(),
          1,
          text.into(),
        ),
      };

      let mut workspace = Workspace::default();

      workspace.open(open(uri("justfile"), justfile)).unwrap();

      let file = tempdir.path().join(path);

      fs::create_dir_all(file.parent().unwrap()).unwrap();
      fs::write(file, "").unwrap();

      workspace.open(open(uri(path), "")).unwrap();

      assert_eq!(
        workspace.links(&uri("justfile")).unwrap(),
        [lsp::DocumentLink {
          range,
          target: Some(uri(path)),
          tooltip: Some(
            uri(path).to_file_path().unwrap().display().to_string()
          ),
          data: None,
        }],
      );
    }

    case("mod foo\n", "foo.just", lsp::Range::at(0, 4, 0, 7));
    case("mod foo\n", "foo/Justfile", lsp::Range::at(0, 4, 0, 7));
    case(
      "mod foo 'bar'\n",
      "bar/mod.just",
      lsp::Range::at(0, 8, 0, 13),
    );
  }

  #[test]
  fn opening_import_discovers_parent_root() {
    let directory = if cfg!(windows) {
      PathBuf::from("C:\\")
    } else {
      PathBuf::from("/")
    };

    let uri =
      |path: &str| lsp::Url::from_file_path(directory.join(path)).unwrap();

    let mut workspace = Workspace {
      documents: DocumentStore::new(Arc::new(MemoryFilesystem::new(
        [("JUSTFILE", "import 'foo/bar.just'"), ("foo/justfile", "")]
          .map(|(path, content)| (directory.join(path), content.into())),
      ))),
      ..Workspace::default()
    };

    workspace
      .open(lsp::DidOpenTextDocumentParams {
        text_document: lsp::TextDocumentItem::new(
          uri("foo/bar.just"),
          "just".into(),
          1,
          String::new(),
        ),
      })
      .unwrap();

    assert_eq!(
      workspace
        .root_projects()
        .into_iter()
        .map(|project| &project.root)
        .collect::<Vec<_>>(),
      [&uri("JUSTFILE")],
    );
  }

  #[test]
  fn project_configuration_applies_to_imports() {
    let tempdir = tempfile::tempdir().unwrap();

    for (path, content) in [
      (
        "just-lsp.toml",
        "[rules]\nunresolved-dependency = 'warning'",
      ),
      (
        "foo/just-lsp.toml",
        "[rules]\nunresolved-dependency = 'off'",
      ),
      ("foo/justfile", "import '../bar/justfile'\nfoo:\n"),
      (
        "bar/just-lsp.toml",
        "[rules]\nunresolved-dependency = 'error'",
      ),
      ("bar/justfile", "bar: baz\n"),
      ("baz/justfile", "foo: bar\n"),
    ] {
      let path = tempdir.path().join(path);

      fs::create_dir_all(path.parent().unwrap()).unwrap();
      fs::write(path, content).unwrap();
    }

    let uri =
      |path| lsp::Url::from_file_path(tempdir.path().join(path)).unwrap();

    let foo = uri("foo/justfile");
    let bar = uri("bar/justfile");
    let baz = uri("baz/justfile");

    let mut workspace = Workspace::default();

    for uri in [&foo, &bar, &baz] {
      workspace.load_project(uri.clone()).unwrap();
    }

    assert_eq!(
      workspace.document_config(&bar),
      Some(Config {
        formatting: FormattingConfig::default(),
        rules: HashMap::from([(
          "unresolved-dependency".into(),
          RuleConfig::Level(RuleLevel::Off),
        )]),
      }),
    );

    assert_eq!(
      workspace
        .diagnostics()
        .into_iter()
        .map(|(uri, diagnostics)| (
          uri,
          diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.severity)
            .collect::<Vec<_>>(),
        ))
        .collect::<BTreeMap<_, _>>(),
      BTreeMap::from([
        (foo, Vec::new()),
        (bar, Vec::new()),
        (baz, vec![lsp::DiagnosticSeverity::WARNING]),
      ]),
    );
  }

  #[test]
  fn project_configuration_applies_to_modules() {
    let tempdir = tempfile::tempdir().unwrap();

    for (path, content) in [
      (
        "just-lsp.toml",
        "[rules]\nunresolved-dependency = 'warning'",
      ),
      (
        "foo/just-lsp.toml",
        "[rules]\nunresolved-dependency = 'off'",
      ),
      ("justfile", "mod foo\n"),
      ("foo/justfile", "foo: bar\n"),
    ] {
      let path = tempdir.path().join(path);

      fs::create_dir_all(path.parent().unwrap()).unwrap();
      fs::write(path, content).unwrap();
    }

    let uri =
      |path| lsp::Url::from_file_path(tempdir.path().join(path)).unwrap();

    let mut workspace = Workspace::default();

    for uri in [uri("justfile"), uri("foo/justfile")] {
      workspace.load_project(uri).unwrap();
    }

    assert_eq!(
      workspace.document_config(&uri("foo/justfile")),
      Some(Config {
        formatting: FormattingConfig::default(),
        rules: HashMap::from([(
          "unresolved-dependency".into(),
          RuleConfig::Level(RuleLevel::Warning),
        )]),
      }),
    );
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
