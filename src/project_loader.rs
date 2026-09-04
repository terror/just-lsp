use super::*;

pub struct ProjectLoader<'a> {
  active: HashSet<lsp::Url>,
  documents: &'a mut DocumentStore,
  expanded: HashSet<lsp::Url>,
  project: Project,
}

impl<'a> ProjectLoader<'a> {
  fn add_dependency(&mut self, source: &lsp::Url, import: Import) -> Result {
    let target = self.resolve_dependency_target(source, &import)?;

    let dependency = ProjectDependency {
      kind: ProjectDependencyKind::Import {
        attributes: import.attributes,
        optional: import.optional,
      },
      location: import.path.range,
      target,
    };

    self.project.add_dependency(source, dependency);

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
      expanded: HashSet::new(),
      project: Project::new(root.clone()),
    };

    loader.visit(root)?;

    loader.project.build_import_scope();

    Ok(loader.project)
  }

  fn resolve_dependency_target(
    &mut self,
    source: &lsp::Url,
    import: &Import,
  ) -> Result<ProjectDependencyTarget> {
    if import.is_dynamic() {
      return Ok(ProjectDependencyTarget::Dynamic);
    }

    let path = match import.resolve(source) {
      Ok(Some(path)) => path,
      Ok(None) | Err(Error::EmptyImportPath) => {
        return Ok(ProjectDependencyTarget::Missing);
      }
      Err(_) => return Ok(ProjectDependencyTarget::Dynamic),
    };

    let path = path.as_path().lexiclean();

    let Ok(uri) = lsp::Url::from_file_path(&path) else {
      return Ok(ProjectDependencyTarget::Missing);
    };

    self.project.add_dependent(&uri, source);

    if self.active.contains(&uri) {
      return Ok(ProjectDependencyTarget::Cycle);
    }

    if self.documents.load(&uri).is_err() {
      if !import.optional {
        warn!(path = %path.display(), "failed to read import");
      }

      return Ok(ProjectDependencyTarget::Missing);
    }

    if !self.expanded.contains(&uri) {
      self.visit(&uri)?;
    }

    Ok(ProjectDependencyTarget::Resolved(uri))
  }

  fn visit(&mut self, uri: &lsp::Url) -> Result {
    self.active.insert(uri.clone());

    let imports = self.documents.load(uri)?.imports();

    self.project.dependencies.entry(uri.clone()).or_default();

    for import in imports {
      self.add_dependency(uri, import)?;
    }

    self.active.remove(uri);
    self.expanded.insert(uri.clone());

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
  fn loads_shell_expanded_import() {
    let mut test =
      Test::new("import x'''foo.just'''\n\nbar: foo").file("foo.just", "foo:");

    let imported = test.uri("foo.just");

    let project = test.load();

    assert_eq!(
      project.dependencies[&test.root][0].target,
      ProjectDependencyTarget::Resolved(imported.clone()),
    );

    assert_eq!(
      project
        .imported_documents(&test.documents)
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>(),
      [imported],
    );
  }

  #[test]
  fn import_scope_deduplicates_diamond_imports() {
    let mut test = Test::new("import 'left.just'\nimport 'right.just'")
      .file("left.just", "import 'shared.just'")
      .file("right.just", "import 'shared.just'")
      .file("shared.just", "");

    let left = test.uri("left.just");
    let right = test.uri("right.just");
    let root = test.root.clone();
    let shared = test.uri("shared.just");

    assert_eq!(
      test.load().import_scope.documents(),
      [
        ImportScopeDocument {
          load_depth: 0,
          traversal_order: 0,
          uri: root,
        },
        ImportScopeDocument {
          load_depth: 1,
          traversal_order: 1,
          uri: right,
        },
        ImportScopeDocument {
          load_depth: 2,
          traversal_order: 2,
          uri: shared,
        },
        ImportScopeDocument {
          load_depth: 1,
          traversal_order: 3,
          uri: left,
        },
      ]
    );
  }

  #[test]
  fn import_scope_deduplicates_repeated_imports() {
    let mut test =
      Test::new("import 'foo.just'\nimport 'foo.just'").file("foo.just", "");

    let foo = test.uri("foo.just");
    let root = test.root.clone();

    assert_eq!(
      test.load().import_scope.documents(),
      [
        ImportScopeDocument {
          load_depth: 0,
          traversal_order: 0,
          uri: root,
        },
        ImportScopeDocument {
          load_depth: 1,
          traversal_order: 1,
          uri: foo,
        },
      ]
    );
  }

  #[test]
  fn import_scope_uses_lifo_order_and_first_load_depth() {
    let mut test =
      Test::new("import 'baz.just'\nimport 'foo.just'\nimport 'bar.just'")
        .file("foo.just", "")
        .file("bar.just", "import 'baz.just'")
        .file("baz.just", "");

    let bar = test.uri("bar.just");
    let baz = test.uri("baz.just");
    let foo = test.uri("foo.just");
    let root = test.root.clone();

    assert_eq!(
      test.load().import_scope.documents(),
      [
        ImportScopeDocument {
          load_depth: 0,
          traversal_order: 0,
          uri: root,
        },
        ImportScopeDocument {
          load_depth: 1,
          traversal_order: 1,
          uri: bar,
        },
        ImportScopeDocument {
          load_depth: 1,
          traversal_order: 2,
          uri: foo,
        },
        ImportScopeDocument {
          load_depth: 2,
          traversal_order: 3,
          uri: baz,
        },
      ]
    );
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

  #[test]
  fn loads_import_graph() {
    let mut test = Test::new(indoc! {
      "
      import 'nested/../bar.just'
      import? 'missing.just'
      import 'required-missing.just'
      import 'bar.just'
      import f'dynamic.just'

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
      project.import_scope.documents(),
      [
        ImportScopeDocument {
          load_depth: 0,
          traversal_order: 0,
          uri: test.root.clone(),
        },
        ImportScopeDocument {
          load_depth: 1,
          traversal_order: 1,
          uri: bar.clone(),
        },
        ImportScopeDocument {
          load_depth: 2,
          traversal_order: 2,
          uri: baz.clone(),
        },
      ]
    );

    assert_eq!(
      project
        .imported_documents(&test.documents)
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>(),
      [bar.clone(), baz]
    );

    assert_eq!(project.dependents[&bar], HashSet::from([test.root.clone()]));
    assert_eq!(project.dependents[&test.root], HashSet::from([bar]));
  }
}
