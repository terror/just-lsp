use super::*;

pub struct ProjectLoader<'a> {
  active: HashSet<lsp::Url>,
  documents: &'a mut DocumentStore,
  project: Project,
  visited: HashSet<lsp::Url>,
}

impl<'a> ProjectLoader<'a> {
  fn dependency(&mut self, source: &lsp::Url, import: Import) -> Result {
    let dynamic =
      import.path.value.starts_with('f') || import.path.value.starts_with('x');

    let path = import.resolve(source);

    let kind = ProjectDependencyKind::Import {
      attributes: import.attributes,
      optional: import.optional,
    };

    let target = if dynamic {
      ProjectDependencyTarget::Dynamic
    } else if let Some(path) = path {
      let path = path.as_path().lexiclean();

      if let Ok(uri) = lsp::Url::from_file_path(&path) {
        if self.active.contains(&uri) {
          self
            .project
            .dependents
            .entry(uri)
            .or_default()
            .insert(source.clone());

          ProjectDependencyTarget::Cycle
        } else if self.documents.load(&uri).is_ok() {
          self
            .project
            .dependents
            .entry(uri.clone())
            .or_default()
            .insert(source.clone());

          if !self.visited.contains(&uri) {
            self.visit(&uri)?;
            self.project.imported.push(uri.clone());
          }

          ProjectDependencyTarget::Resolved(uri)
        } else {
          if !import.optional {
            warn!(path = %path.display(), "failed to read import");
          }

          ProjectDependencyTarget::Missing
        }
      } else {
        ProjectDependencyTarget::Missing
      }
    } else {
      ProjectDependencyTarget::Missing
    };

    self
      .project
      .dependencies
      .entry(source.clone())
      .or_default()
      .push(ProjectDependency {
        kind,
        location: import.path.range,
        target,
      });

    Ok(())
  }

  /// # Errors
  ///
  /// Returns an [`Error`] if the root document cannot be loaded.
  pub fn load(
    documents: &'a mut DocumentStore,
    root: &lsp::Url,
  ) -> Result<Project> {
    documents.load(root)?;

    let mut loader = Self {
      active: HashSet::new(),
      documents,
      project: Project {
        dependencies: HashMap::new(),
        dependents: HashMap::new(),
        imported: Vec::new(),
        root: root.clone(),
      },
      visited: HashSet::new(),
    };

    loader.visit(root)?;

    Ok(loader.project)
  }

  fn visit(&mut self, uri: &lsp::Url) -> Result {
    self.active.insert(uri.clone());

    let imports = self.documents.load(uri)?.imports();

    self.project.dependencies.entry(uri.clone()).or_default();

    for import in imports {
      self.dependency(uri, import)?;
    }

    self.active.remove(uri);
    self.visited.insert(uri.clone());

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  struct Test {
    documents: DocumentStore,
    root: lsp::Url,
    tempdir: tempfile::TempDir,
  }

  impl Test {
    fn file(self, path: &str, content: &str) -> Self {
      let path = self.tempdir.path().join(path);

      fs::create_dir_all(path.parent().unwrap()).unwrap();
      fs::write(path, content).unwrap();

      self
    }

    fn load(&mut self) -> Project {
      ProjectLoader::load(&mut self.documents, &self.root).unwrap()
    }

    fn new(content: &str) -> Self {
      let tempdir = tempfile::tempdir().unwrap();
      let root = tempdir.path().join("justfile");

      fs::write(&root, content).unwrap();

      Self {
        documents: DocumentStore::default(),
        root: lsp::Url::from_file_path(root).unwrap(),
        tempdir,
      }
    }

    fn open(&mut self, path: &str, content: &str) {
      self
        .documents
        .open(lsp::DidOpenTextDocumentParams {
          text_document: lsp::TextDocumentItem {
            uri: self.uri(path),
            language_id: "just".into(),
            version: 1,
            text: content.into(),
          },
        })
        .unwrap();
    }

    fn uri(&self, path: &str) -> lsp::Url {
      lsp::Url::from_file_path(self.tempdir.path().join(path)).unwrap()
    }
  }

  #[test]
  fn analyzer_uses_imported_declarations() {
    let mut test =
      Test::new("import 'foo.just'\n\nbar: foo").file("foo.just", "foo:");

    let project = test.load();

    assert!(
      Analyzer {
        config: None,
        document: test.documents.get(&test.root).unwrap(),
        imported_documents: project
          .imported_documents(&test.documents)
          .collect(),
      }
      .analyze()
      .is_empty()
    );
  }

  #[test]
  fn loads_import_graph() {
    let mut test = Test::new(indoc! {
      "
      import 'nested/../bar.just'
      import? 'missing.just'
      import 'required-missing.just'
      import 'bar.just'
      import x'dynamic.just'

      foo:
      "
    })
    .file(
      "bar.just",
      indoc! {
        "
        import 'nested/baz.just'
        import 'justfile'

        bar:
        "
      },
    )
    .file("nested/baz.just", "baz:");

    let bar = test.uri("bar.just");
    let baz = test.uri("nested/baz.just");

    let project = test.load();

    assert_eq!(project.root, test.root);

    assert_eq!(
      project.dependencies[&test.root]
        .iter()
        .map(|dependency| dependency.target.clone())
        .collect::<Vec<_>>(),
      [
        ProjectDependencyTarget::Resolved(bar.clone()),
        ProjectDependencyTarget::Missing,
        ProjectDependencyTarget::Missing,
        ProjectDependencyTarget::Resolved(bar.clone()),
        ProjectDependencyTarget::Dynamic,
      ]
    );

    assert_eq!(
      project.dependencies[&test.root]
        .iter()
        .map(|dependency| match &dependency.kind {
          ProjectDependencyKind::Import { optional, .. } => *optional,
        })
        .collect::<Vec<_>>(),
      [false, true, false, false, false]
    );

    assert_eq!(
      project.dependencies[&bar]
        .iter()
        .map(|dependency| dependency.target.clone())
        .collect::<Vec<_>>(),
      [
        ProjectDependencyTarget::Resolved(baz.clone()),
        ProjectDependencyTarget::Cycle,
      ]
    );

    assert_eq!(
      project
        .imported_documents(&test.documents)
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>(),
      [baz, bar.clone()]
    );

    assert_eq!(project.dependents[&bar], HashSet::from([test.root.clone()]));
    assert_eq!(project.dependents[&test.root], HashSet::from([bar]));
  }

  #[test]
  fn loading_prefers_open_import() {
    let mut test = Test::new("import 'foo.just'").file("foo.just", "disk:");

    test.open("foo.just", "buffer:");

    let project = test.load();

    assert_eq!(
      project
        .imported_documents(&test.documents)
        .flat_map(Document::recipes)
        .map(|recipe| recipe.name.value)
        .collect::<Vec<_>>(),
      ["buffer"]
    );
  }
}
