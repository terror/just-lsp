use super::*;

pub(crate) struct ProjectLoader<'a> {
  ancestors: Vec<lsp::Url>,
  circular_dependencies: HashMap<lsp::Url, Vec<ProjectDependency>>,
  dependencies: HashMap<lsp::Url, Vec<ProjectDependency>>,
  documents: &'a mut DocumentStore,
}

impl<'a> ProjectLoader<'a> {
  /// # Errors
  ///
  /// Returns an [`Error`] if the root document cannot be loaded.
  pub(crate) fn load(
    documents: &'a mut DocumentStore,
    root: &lsp::Url,
  ) -> Result<Project> {
    documents.load(root)?;

    let mut loader = Self {
      ancestors: Vec::new(),
      circular_dependencies: HashMap::new(),
      dependencies: HashMap::new(),
      documents,
    };

    loader.visit(root);

    Ok(Project::new(
      root.clone(),
      loader.dependencies,
      loader.circular_dependencies,
    ))
  }

  fn visit(&mut self, uri: &lsp::Url) {
    let dependencies = self.documents.dependencies(uri).unwrap_or_default();

    self.dependencies.insert(uri.clone(), dependencies.clone());

    self.ancestors.push(uri.clone());

    for dependency in dependencies {
      let ProjectDependencyTarget::Path(target) = &dependency.target else {
        continue;
      };

      if self.ancestors.contains(target) {
        self
          .circular_dependencies
          .entry(uri.clone())
          .or_default()
          .push(dependency);

        continue;
      }

      if self.dependencies.contains_key(target) {
        continue;
      }

      if self.documents.load(target).is_err() {
        if !dependency.optional {
          warn!(%target, "failed to read dependency");
        }

        continue;
      }

      self.visit(target);
    }

    self.ancestors.pop();
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
  fn conditional_import_is_disabled() {
    let disabled = if cfg!(windows) { "unix" } else { "windows" };

    let mut test = Test::new(&format!("[{disabled}]\nimport 'foo.just'"))
      .file("foo.just", "import 'bar.just'")
      .file("bar.just", "bar:");

    let project = test.load();

    assert_eq!(
      project.module().import_scope.documents(),
      [ScopeDocument {
        load_depth: 0,
        document: test.root.clone(),
      }],
    );

    assert_eq!(
      project.dependencies,
      HashMap::from([(test.root.clone(), Vec::new())]),
    );

    assert_eq!(project.targets, HashSet::new());
    assert!(test.documents.get(&test.uri("foo.just")).is_none());
    assert!(test.documents.get(&test.uri("bar.just")).is_none());
  }

  #[test]
  fn conditional_import_is_enabled() {
    let mut test =
      Test::new(&format!("[{}]\nimport 'foo.just'", env::consts::OS))
        .file("foo.just", "import 'bar.just'")
        .file("bar.just", "bar:");

    let project = test.load();

    assert_eq!(
      project.module().import_scope.documents(),
      [
        ScopeDocument {
          load_depth: 0,
          document: test.root.clone(),
        },
        ScopeDocument {
          load_depth: 1,
          document: test.uri("foo.just"),
        },
        ScopeDocument {
          load_depth: 2,
          document: test.uri("bar.just"),
        },
      ],
    );
  }

  #[test]
  fn conditional_import_with_matching_attribute_is_enabled() {
    let disabled = if cfg!(windows) { "unix" } else { "windows" };

    let mut test = Test::new(&format!(
      "[{disabled}, {}]\nimport 'foo.just'",
      env::consts::OS,
    ))
    .file("foo.just", "foo:");

    let project = test.load();

    assert_eq!(
      project.module().import_scope.documents(),
      [
        ScopeDocument {
          load_depth: 0,
          document: test.root.clone(),
        },
        ScopeDocument {
          load_depth: 1,
          document: test.uri("foo.just"),
        },
      ],
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
      test.load().module().import_scope.documents(),
      [
        ScopeDocument {
          load_depth: 0,
          document: root,
        },
        ScopeDocument {
          load_depth: 1,
          document: right,
        },
        ScopeDocument {
          load_depth: 2,
          document: shared,
        },
        ScopeDocument {
          load_depth: 1,
          document: left,
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
      test.load().module().import_scope.documents(),
      [
        ScopeDocument {
          load_depth: 0,
          document: root,
        },
        ScopeDocument {
          load_depth: 1,
          document: foo,
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
      test.load().module().import_scope.documents(),
      [
        ScopeDocument {
          load_depth: 0,
          document: root,
        },
        ScopeDocument {
          load_depth: 1,
          document: bar,
        },
        ScopeDocument {
          load_depth: 1,
          document: foo,
        },
        ScopeDocument {
          load_depth: 2,
          document: baz,
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
        .module()
        .import_scope
        .documents()
        .iter()
        .skip(1)
        .filter_map(|document| test.documents.get(&document.document))
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
        ProjectDependencyTarget::Path(bar.clone()),
        ProjectDependencyTarget::Path(test.uri("missing.just")),
        ProjectDependencyTarget::Path(test.uri("required-missing.just")),
        ProjectDependencyTarget::Path(bar.clone()),
        ProjectDependencyTarget::Dynamic,
      ]
    );

    assert_eq!(
      project.dependencies[&bar]
        .iter()
        .map(|dependency| dependency.target.clone())
        .collect::<Vec<_>>(),
      [
        ProjectDependencyTarget::Path(baz.clone()),
        ProjectDependencyTarget::Path(test.root.clone()),
      ]
    );

    assert_eq!(
      project.module().import_scope.documents(),
      [
        ScopeDocument {
          load_depth: 0,
          document: test.root.clone(),
        },
        ScopeDocument {
          load_depth: 1,
          document: bar.clone(),
        },
        ScopeDocument {
          load_depth: 2,
          document: baz.clone(),
        },
      ]
    );

    assert_eq!(
      project.targets,
      HashSet::from([
        bar,
        baz,
        test.root.clone(),
        test.uri("missing.just"),
        test.uri("required-missing.just"),
      ]),
    );
  }

  #[test]
  fn loads_modules() {
    let mut test = Test::new("mod foo")
      .file("foo.just", "import 'bar.just'")
      .file("bar.just", "bar:");

    let foo = test.uri("foo.just");
    let bar = test.uri("bar.just");

    let project = test.load();

    assert_eq!(
      project.dependencies[&test.root],
      [ProjectDependency {
        kind: ProjectDependencyKind::Module { name: "foo".into() },
        location: lsp::Range::at(0, 4, 0, 7),
        optional: false,
        target: ProjectDependencyTarget::Path(foo.clone()),
      }],
    );

    assert_eq!(
      project.dependencies.keys().collect::<HashSet<_>>(),
      HashSet::from([&test.root, &foo, &bar]),
    );

    assert_eq!(
      project.module().import_scope.documents(),
      [ScopeDocument {
        document: test.root.clone(),
        load_depth: 0,
      }],
    );
  }

  #[test]
  fn loads_shared_modules_once() {
    let mut test =
      Test::new("mod foo 'bar.just'\nmod baz 'bar.just'").file("bar.just", "");

    let bar = test.uri("bar.just");

    let project = test.load();

    assert_eq!(
      project.modules.keys().collect::<HashSet<_>>(),
      HashSet::from([&test.root, &bar]),
    );

    assert_eq!(
      project.module().children,
      BTreeMap::from([("baz".into(), bar.clone()), ("foo".into(), bar)]),
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
      ProjectDependencyTarget::Path(imported.clone()),
    );

    assert_eq!(
      project.module().import_scope.documents(),
      [
        ScopeDocument {
          document: test.root.clone(),
          load_depth: 0,
        },
        ScopeDocument {
          document: imported,
          load_depth: 1,
        },
      ],
    );
  }
}
