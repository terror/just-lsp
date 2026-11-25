use {
  super::*, imported_scope::ImportedScope, local_scope::LocalScope,
  module_scope::ModuleScope,
};

mod imported_scope;
mod local_scope;
mod module_scope;

pub(crate) struct RuleContext<'a> {
  attributes: OnceLock<Vec<Attribute>>,
  function_calls: OnceLock<Vec<FunctionCall>>,
  usage: OnceLock<Arc<Usage>>,
  view: &'a ModuleView<'a>,
}

impl<'a> RuleContext<'a> {
  pub(crate) fn bindings(&self, document: &Document) -> Arc<Bindings> {
    self.view.bindings(document)
  }

  pub(crate) fn circular_dependencies(&self) -> &'a [ProjectDependency] {
    self.view.circular_dependencies()
  }

  pub(super) fn conflicting_recipe_attributes(
    &self,
    left: &str,
    right: &str,
  ) -> Vec<(&Recipe, &Attribute)> {
    self
      .local()
      .recipes()
      .filter_map(|recipe| {
        let attribute = recipe.find_attribute(left)?;

        recipe.find_attribute(right)?;

        Some((&**recipe, attribute))
      })
      .collect()
  }

  pub(super) fn conflicting_settings(
    &self,
    incompatible: impl Fn(&Setting, &Setting) -> bool,
  ) -> Vec<(&Located<Setting>, &Located<Setting>)> {
    let mut settings = self
      .module()
      .settings()
      .iter()
      .map(|setting| {
        let groups = GroupSet::from_attributes(&setting.attributes);

        (setting, groups)
      })
      .collect::<Vec<_>>();

    settings.sort_by_key(|(setting, _)| self.is_local(setting));

    let mut conflicts = Vec::new();

    for (index, (current, current_groups)) in settings.iter().enumerate() {
      if !self.is_local(current) {
        continue;
      }

      let mut seen = HashSet::new();

      for (previous, previous_groups) in &settings[..index] {
        if previous_groups.conflicts_with(current_groups)
          && incompatible(previous, current)
          && seen.insert(previous.name.value.as_str())
        {
          conflicts.push((*previous, *current));
        }
      }
    }

    conflicts
  }

  pub(crate) fn dependencies(
    &self,
  ) -> impl Iterator<Item = &'a ProjectDependency> + '_ {
    self.view.dependencies()
  }

  pub(crate) fn document(&self) -> &'a Document {
    self.view.document()
  }

  pub(crate) fn documents(&self) -> impl Iterator<Item = &'a Document> + '_ {
    self.view.documents()
  }

  pub(super) fn duplicate_declarations<'b, T>(
    &self,
    declarations: &'b [Located<T>],
    name: impl Fn(&T) -> &TextNode,
    attributes: impl Fn(&T) -> &[Attribute],
  ) -> Vec<&'b T> {
    let mut conflicts = ConflictTracker::default();

    for declaration in declarations {
      if !self.is_local(declaration) {
        conflicts.record(name(declaration), attributes(declaration));
      }
    }

    declarations
      .iter()
      .filter(|declaration| self.is_local(declaration))
      .map(Deref::deref)
      .filter(|declaration| {
        conflicts.record(name(declaration), attributes(declaration))
      })
      .collect()
  }

  pub(crate) fn filesystem(&self) -> &'a dyn Filesystem {
    self.view.filesystem()
  }

  pub(crate) fn imported(&self) -> ImportedScope<'_> {
    ImportedScope::new(self)
  }

  fn is_local<T>(&self, declaration: &Located<T>) -> bool {
    declaration.uri == self.document().uri
  }

  pub(crate) fn loads(&self, path: &Path) -> bool {
    self.view.loads(path)
  }

  pub(crate) fn local(&self) -> LocalScope<'_> {
    LocalScope::new(self)
  }

  pub(crate) fn module(&self) -> ModuleScope<'_> {
    ModuleScope::new(self)
  }

  #[must_use]
  pub(crate) fn new(view: &'a ModuleView<'a>) -> Self {
    Self {
      attributes: OnceLock::new(),
      function_calls: OnceLock::new(),
      usage: OnceLock::new(),
      view,
    }
  }

  fn usage(&self) -> &Usage {
    self.usage.get_or_init(|| self.view.usage())
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*, indoc::indoc, pretty_assertions::assert_eq, tempfile::Builder,
  };

  fn context(path: &Path, test: impl FnOnce(&RuleContext<'_>)) {
    let uri = lsp::Url::from_file_path(path).unwrap();

    let mut documents = DocumentStore::default();

    let project = ProjectLoader::load(&mut documents, &uri).unwrap();

    test(&RuleContext::new(&ModuleView::new(
      documents.get(&uri).unwrap(),
      ModuleContext {
        documents: &documents,
        module: project.module(),
        project: &project,
      },
    )));
  }

  #[test]
  fn analyzed_document_is_selected_by_uri() {
    let root =
      Document::new("foo:\n", lsp::Url::parse("file:///foo.just").unwrap())
        .unwrap();

    let document =
      Document::new("foo:\n", lsp::Url::parse("file:///bar.just").unwrap())
        .unwrap();

    let view = ModuleView::from_scope(
      &document,
      vec![
        ScopeDocument {
          document: &root,
          load_depth: 0,
        },
        ScopeDocument {
          document: &document,
          load_depth: 1,
        },
      ],
    );

    let context = RuleContext::new(&view);

    assert_eq!(
      context
        .module()
        .recipes()
        .iter()
        .map(|recipe| recipe.location(recipe.name.range))
        .collect::<Vec<_>>(),
      [
        lsp::Location::new(root.uri.clone(), lsp::Range::at(0, 0, 0, 3)),
        lsp::Location::new(document.uri.clone(), lsp::Range::at(0, 0, 0, 3)),
      ],
    );

    assert_eq!(
      context
        .local()
        .recipes()
        .map(|recipe| recipe.location(recipe.name.range))
        .collect::<Vec<_>>(),
      [lsp::Location::new(
        document.uri.clone(),
        lsp::Range::at(0, 0, 0, 3)
      )],
    );

    assert_eq!(
      context
        .imported()
        .recipes()
        .map(|recipe| recipe.location(recipe.name.range))
        .collect::<Vec<_>>(),
      [lsp::Location::new(
        root.uri.clone(),
        lsp::Range::at(0, 0, 0, 3)
      )],
    );
  }

  #[test]
  fn circular_imports_are_handled() {
    let dir = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::write(
      dir.path().join("bar.just"),
      indoc! {
        "
        import 'justfile'

        bar:
          echo bar
        "
      },
    )
    .unwrap();

    fs::write(
      dir.path().join("justfile"),
      indoc! {
        "
        import 'bar.just'

        foo:
          echo foo
        "
      },
    )
    .unwrap();

    context(&dir.path().join("justfile"), |context| {
      let recipe_names = context
        .module()
        .recipes()
        .iter()
        .map(|recipe| recipe.name.value.as_str())
        .collect::<Vec<_>>();

      assert_eq!(recipe_names, ["foo", "bar"]);
    });
  }

  #[test]
  fn imported_recipes_are_merged() {
    let dir = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::write(
      dir.path().join("bar.just"),
      indoc! {
        "
        bar:
          echo bar
        "
      },
    )
    .unwrap();

    fs::write(
      dir.path().join("justfile"),
      indoc! {
        "
        import 'bar.just'

        foo:
          echo foo
        "
      },
    )
    .unwrap();

    context(&dir.path().join("justfile"), |context| {
      let recipe_names = context
        .module()
        .recipes()
        .iter()
        .map(|recipe| recipe.name.value.as_str())
        .collect::<Vec<_>>();

      assert_eq!(recipe_names, ["foo", "bar"]);
    });
  }

  #[test]
  fn imported_settings_are_merged() {
    let dir = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::write(dir.path().join("bar.just"), "set export\n").unwrap();

    fs::write(
      dir.path().join("justfile"),
      indoc! {
        "
        import 'bar.just'

        set dotenv-load
        "
      },
    )
    .unwrap();

    context(&dir.path().join("justfile"), |context| {
      let setting_names = context
        .module()
        .settings()
        .iter()
        .map(|s| s.name.value.as_str())
        .collect::<Vec<_>>();

      assert_eq!(setting_names, ["dotenv-load", "export"]);
    });
  }

  #[test]
  fn imported_variables_are_merged() {
    let dir = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::write(dir.path().join("bar.just"), "bar := 'baz'\n").unwrap();

    fs::write(
      dir.path().join("justfile"),
      indoc! {
        "
        import 'bar.just'

        foo := 'qux'
        "
      },
    )
    .unwrap();

    context(&dir.path().join("justfile"), |context| {
      let variable_names = context
        .module()
        .variables()
        .iter()
        .map(|variable| variable.name.value.as_str())
        .collect::<Vec<_>>();

      assert_eq!(variable_names, ["foo", "bar"]);
    });
  }

  #[test]
  fn optional_missing_import_is_skipped() {
    let dir = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::write(
      dir.path().join("justfile"),
      indoc! {
        "
        import? 'nonexistent.just'

        foo:
          echo foo
        "
      },
    )
    .unwrap();

    context(&dir.path().join("justfile"), |context| {
      let recipe_names = context
        .module()
        .recipes()
        .iter()
        .map(|recipe| recipe.name.value.as_str())
        .collect::<Vec<_>>();

      assert_eq!(recipe_names, ["foo"]);
    });
  }

  #[test]
  fn recursive_imports_are_resolved() {
    let dir = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::write(
      dir.path().join("baz.just"),
      indoc! {
        "
        baz:
          echo baz
        "
      },
    )
    .unwrap();

    fs::write(
      dir.path().join("bar.just"),
      indoc! {
        "
        import 'baz.just'

        bar:
          echo bar
        "
      },
    )
    .unwrap();

    fs::write(
      dir.path().join("justfile"),
      indoc! {
        "
        import 'bar.just'

        foo:
          echo foo
        "
      },
    )
    .unwrap();

    context(&dir.path().join("justfile"), |context| {
      let recipe_names = context
        .module()
        .recipes()
        .iter()
        .map(|recipe| recipe.name.value.as_str())
        .collect::<Vec<_>>();

      assert_eq!(recipe_names, ["foo", "bar", "baz"]);
    });
  }
}
