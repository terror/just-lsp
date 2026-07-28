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

  #[allow(clippy::missing_errors_doc)]
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

  fn uri(path: &Path) -> lsp::Url {
    lsp::Url::from_file_path(path).unwrap()
  }

  #[test]
  fn analyzer_uses_imported_declarations() {
    let tempdir = tempfile::tempdir().unwrap();
    let root = tempdir.path().join("justfile");

    fs::write(&root, "import 'foo.just'\n\nbar: foo").unwrap();
    fs::write(tempdir.path().join("foo.just"), "foo:").unwrap();

    let root = uri(&root);
    let mut documents = DocumentStore::default();
    let project = ProjectLoader::load(&mut documents, &root).unwrap();
    let document = documents.get(&root).unwrap();

    assert!(
      Analyzer {
        config: None,
        document,
        imported_documents: project.imported_documents(&documents).collect(),
      }
      .analyze()
      .is_empty()
    );
  }

  #[test]
  fn loads_import_graph() {
    let tempdir = tempfile::tempdir().unwrap();
    let root = tempdir.path().join("justfile");
    let bar = tempdir.path().join("bar.just");
    let baz = tempdir.path().join("nested/baz.just");

    fs::create_dir(tempdir.path().join("nested")).unwrap();
    fs::write(&baz, "baz:").unwrap();
    fs::write(
      &bar,
      indoc! {
        "
        import 'nested/baz.just'
        import 'justfile'

        bar:
        "
      },
    )
    .unwrap();
    fs::write(
      &root,
      indoc! {
        "
        import 'nested/../bar.just'
        import? 'missing.just'
        import 'required-missing.just'
        import 'bar.just'
        import x'dynamic.just'

        foo:
        "
      },
    )
    .unwrap();

    let root = uri(&root);
    let bar = uri(&bar);
    let baz = uri(&baz);
    let mut documents = DocumentStore::default();
    let project = ProjectLoader::load(&mut documents, &root).unwrap();

    assert_eq!(project.root, root);

    let root_dependencies = &project.dependencies[&root];

    assert_eq!(root_dependencies.len(), 5);
    assert_eq!(
      root_dependencies[0].target,
      ProjectDependencyTarget::Resolved(bar.clone())
    );
    assert_eq!(
      root_dependencies[1].target,
      ProjectDependencyTarget::Missing
    );
    assert_eq!(
      root_dependencies[1].kind,
      ProjectDependencyKind::Import {
        attributes: Vec::new(),
        optional: true,
      }
    );
    assert_eq!(
      root_dependencies[2].target,
      ProjectDependencyTarget::Missing
    );
    assert_eq!(
      root_dependencies[2].kind,
      ProjectDependencyKind::Import {
        attributes: Vec::new(),
        optional: false,
      }
    );
    assert_eq!(
      root_dependencies[3].target,
      ProjectDependencyTarget::Resolved(bar.clone())
    );
    assert_eq!(
      root_dependencies[4].target,
      ProjectDependencyTarget::Dynamic
    );

    assert_eq!(
      project.dependencies[&bar]
        .iter()
        .map(|dependency| dependency.target.clone())
        .collect::<Vec<_>>(),
      [
        ProjectDependencyTarget::Resolved(baz.clone()),
        ProjectDependencyTarget::Cycle
      ]
    );

    assert_eq!(
      project
        .imported_documents(&documents)
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>(),
      [baz, bar.clone()]
    );

    assert_eq!(project.dependents[&bar], HashSet::from([root.clone()]));
    assert_eq!(project.dependents[&root], HashSet::from([bar]));
  }

  #[test]
  fn loading_prefers_open_import() {
    let tempdir = tempfile::tempdir().unwrap();
    let root = tempdir.path().join("justfile");
    let imported = tempdir.path().join("foo.just");

    fs::write(&root, "import 'foo.just'").unwrap();
    fs::write(&imported, "disk:").unwrap();

    let imported = uri(&imported);
    let mut documents = DocumentStore::default();

    documents
      .open(lsp::DidOpenTextDocumentParams {
        text_document: lsp::TextDocumentItem {
          uri: imported,
          language_id: "just".into(),
          version: 1,
          text: "buffer:".into(),
        },
      })
      .unwrap();

    let project = ProjectLoader::load(&mut documents, &uri(&root)).unwrap();

    assert_eq!(
      project
        .imported_documents(&documents)
        .flat_map(Document::recipes)
        .map(|recipe| recipe.name.value)
        .collect::<Vec<_>>(),
      ["buffer"]
    );
  }
}
