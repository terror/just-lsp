use super::*;

#[derive(Clone, Debug)]
pub struct ModuleView<'a> {
  context: Option<ModuleContext<'a>>,
  document: &'a Document,
  documents: Vec<ScopeDocument<&'a Document>>,
  symbols: Arc<SymbolTable>,
}

impl<'a> ModuleView<'a> {
  pub(crate) fn aliases(&self) -> &[Located<Alias>] {
    &self.symbols.aliases
  }

  #[must_use]
  pub fn child(&self, name: &str) -> Option<Self> {
    let context = self.context?;

    let root = context.module.children.get(name)?;

    Some(Self::new(
      context.documents.get(root)?,
      ModuleContext {
        module: context.project.modules.get(root)?,
        ..context
      },
    ))
  }

  #[must_use]
  pub fn document(&self) -> &'a Document {
    self.document
  }

  pub(crate) fn documents(&self) -> impl Iterator<Item = &'a Document> + '_ {
    self.documents.iter().map(|document| document.document)
  }

  #[must_use]
  pub fn find_function(&self, name: &str) -> Option<&Located<Function>> {
    self
      .symbols
      .resolved_functions
      .get(name)
      .map(|index| &self.symbols.functions[*index])
  }

  #[must_use]
  pub fn find_recipe(&self, name: &str) -> Option<&Located<Recipe>> {
    self
      .symbols
      .resolved_recipes
      .get(name)
      .map(|index| &self.symbols.recipes[*index])
  }

  #[must_use]
  pub fn find_variable(&self, name: &str) -> Option<&Located<Variable>> {
    self
      .symbols
      .resolved_variables
      .get(name)
      .map(|index| &self.symbols.variables[*index])
  }

  pub(crate) fn from_scope(
    document: &'a Document,
    documents: Vec<ScopeDocument<&'a Document>>,
  ) -> Self {
    Self {
      context: None,
      symbols: Arc::new(SymbolTable::new(&documents)),
      document,
      documents,
    }
  }

  pub(crate) fn functions(&self) -> &[Located<Function>] {
    &self.symbols.functions
  }

  #[must_use]
  pub(crate) fn new(
    document: &'a Document,
    context: ModuleContext<'a>,
  ) -> Self {
    let scope = context
      .module
      .import_scope
      .documents()
      .iter()
      .filter_map(|scope_document| {
        let scoped_document = if scope_document.document == document.uri {
          document
        } else {
          context.documents.get(&scope_document.document)?
        };

        Some(ScopeDocument {
          document: scoped_document,
          load_depth: scope_document.load_depth,
        })
      })
      .collect::<Vec<_>>();

    Self {
      context: Some(context),
      symbols: context.module.symbols(&scope),
      document,
      documents: scope,
    }
  }

  pub(crate) fn recipes(&self) -> &[Located<Recipe>] {
    &self.symbols.recipes
  }

  pub fn resolved_functions(&self) -> impl Iterator<Item = &Located<Function>> {
    self
      .symbols
      .resolved_functions
      .values()
      .map(|index| &self.symbols.functions[*index])
  }

  pub fn resolved_recipes(&self) -> impl Iterator<Item = &Located<Recipe>> {
    self
      .symbols
      .resolved_recipes
      .values()
      .map(|index| &self.symbols.recipes[*index])
  }

  pub fn resolved_variables(&self) -> impl Iterator<Item = &Located<Variable>> {
    self
      .symbols
      .resolved_variables
      .values()
      .map(|index| &self.symbols.variables[*index])
  }

  pub(crate) fn settings(&self) -> &[Located<Setting>] {
    &self.symbols.settings
  }

  pub(crate) fn unexports(&self) -> &[Located<Unexport>] {
    &self.symbols.unexports
  }

  pub(crate) fn variables(&self) -> &[Located<Variable>] {
    &self.symbols.variables
  }

  pub(super) fn with_document(&self, document: &'a Document) -> Self {
    Self {
      document,
      ..self.clone()
    }
  }

  #[must_use]
  pub fn with_imports(
    document: &'a Document,
    imports: impl IntoIterator<Item = &'a Document>,
  ) -> Self {
    Self::from_scope(
      document,
      once(document)
        .chain(imports)
        .enumerate()
        .map(|(index, document)| ScopeDocument {
          document,
          load_depth: usize::from(index > 0),
        })
        .collect(),
    )
  }
}

impl<'a> From<&'a Document> for ModuleView<'a> {
  fn from(document: &'a Document) -> Self {
    Self::from_scope(
      document,
      vec![ScopeDocument {
        document,
        load_depth: 0,
      }],
    )
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  #[track_caller]
  fn load(
    files: &[(&str, &str)],
  ) -> (tempfile::TempDir, DocumentStore, Project) {
    let tempdir = tempfile::tempdir().unwrap();

    for (path, content) in files {
      fs::write(tempdir.path().join(path), content).unwrap();
    }

    let root =
      lsp::Url::from_file_path(tempdir.path().join("justfile")).unwrap();

    let mut documents = DocumentStore::default();

    let project = ProjectLoader::load(&mut documents, &root).unwrap();

    (tempdir, documents, project)
  }

  #[test]
  fn child_views_module_scope() {
    let (tempdir, documents, project) = load(&[
      ("justfile", "mod foo"),
      ("foo.just", "import 'bar.just'"),
      ("bar.just", "baz:"),
    ]);

    let uri =
      |path: &str| lsp::Url::from_file_path(tempdir.path().join(path)).unwrap();

    let view = ModuleView::new(
      documents.get(&project.root).unwrap(),
      ModuleContext {
        documents: &documents,
        module: project.module(),
        project: &project,
      },
    );

    assert!(view.find_recipe("baz").is_none());
    assert!(view.child("bar").is_none());

    let child = view.child("foo").unwrap();

    assert_eq!(child.document().uri, uri("foo.just"));
    assert_eq!(child.find_recipe("baz").unwrap().uri, uri("bar.just"));
  }

  #[test]
  fn direct_import_overrides_nested_import() {
    let root =
      Document::new("", lsp::Url::parse("file:///justfile").unwrap()).unwrap();

    let direct = Document::new(
      indoc! {
        "
        foo := 'foo'
        foo() := 'foo'
        foo:
          echo foo
        "
      },
      lsp::Url::parse("file:///foo.just").unwrap(),
    )
    .unwrap();

    let nested = Document::new(
      indoc! {
        "
        foo := 'bar'
        foo() := 'bar'
        foo:
          echo bar
        "
      },
      lsp::Url::parse("file:///bar.just").unwrap(),
    )
    .unwrap();

    let view = ModuleView::from_scope(
      &root,
      vec![
        ScopeDocument {
          document: &root,
          load_depth: 0,
        },
        ScopeDocument {
          document: &direct,
          load_depth: 1,
        },
        ScopeDocument {
          document: &nested,
          load_depth: 2,
        },
      ],
    );

    assert_eq!(view.find_recipe("foo").unwrap().uri, direct.uri);
    assert_eq!(view.find_variable("foo").unwrap().uri, direct.uri);
    assert_eq!(view.find_function("foo").unwrap().uri, direct.uri);
  }

  #[test]
  fn equal_depth_imports_use_lifo_precedence() {
    let root =
      Document::new("", lsp::Url::parse("file:///justfile").unwrap()).unwrap();

    let first = Document::new(
      indoc! {
        "
        foo := 'foo'
        foo() := 'foo'
        foo:
          echo foo
        "
      },
      lsp::Url::parse("file:///foo.just").unwrap(),
    )
    .unwrap();

    let second = Document::new(
      indoc! {
        "
        foo := 'bar'
        foo() := 'bar'
        foo:
          echo bar
        "
      },
      lsp::Url::parse("file:///bar.just").unwrap(),
    )
    .unwrap();

    let view = ModuleView::from_scope(
      &root,
      vec![
        ScopeDocument {
          document: &root,
          load_depth: 0,
        },
        ScopeDocument {
          document: &second,
          load_depth: 1,
        },
        ScopeDocument {
          document: &first,
          load_depth: 1,
        },
      ],
    );

    assert_eq!(view.find_recipe("foo").unwrap().uri, first.uri);
    assert_eq!(view.find_variable("foo").unwrap().uri, first.uri);
    assert_eq!(view.find_function("foo").unwrap().uri, first.uri);
  }

  #[test]
  fn later_declaration_in_document_wins() {
    let root = Document::new(
      indoc! {
        "
        foo := 'foo'
        foo() := 'foo'
        foo:
          echo foo

        foo := 'bar'
        foo() := 'bar'
        foo:
          echo bar
        "
      },
      lsp::Url::parse("file:///justfile").unwrap(),
    )
    .unwrap();

    let view = ModuleView::from(&root);

    assert_eq!(view.find_recipe("foo").unwrap().content, "foo:\n  echo bar",);
    assert_eq!(view.find_variable("foo").unwrap().content, "foo := 'bar'",);
    assert_eq!(view.find_function("foo").unwrap().content, "foo() := 'bar'",);
  }

  #[test]
  fn root_overrides_imported_declarations() {
    let root = Document::new(
      indoc! {
        "
        foo := 'foo'
        foo() := 'foo'
        foo:
          echo foo
        "
      },
      lsp::Url::parse("file:///justfile").unwrap(),
    )
    .unwrap();

    let imported = Document::new(
      indoc! {
        "
        foo := 'bar'
        foo() := 'bar'
        foo:
          echo bar
        "
      },
      lsp::Url::parse("file:///foo.just").unwrap(),
    )
    .unwrap();

    let view = ModuleView::from_scope(
      &root,
      vec![
        ScopeDocument {
          document: &root,
          load_depth: 0,
        },
        ScopeDocument {
          document: &imported,
          load_depth: 1,
        },
      ],
    );

    assert_eq!(view.find_recipe("foo").unwrap().uri, root.uri);
    assert_eq!(view.find_variable("foo").unwrap().uri, root.uri);
    assert_eq!(view.find_function("foo").unwrap().uri, root.uri);
  }
}
