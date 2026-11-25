use super::*;

pub(crate) struct RuleContext<'a> {
  attributes: OnceLock<Vec<Attribute>>,
  function_calls: OnceLock<Vec<FunctionCall>>,
  usage: OnceLock<Arc<Usage>>,
  view: &'a ModuleView<'a>,
}

impl<'a> RuleContext<'a> {
  pub(crate) fn aliases(&self) -> &[Located<Alias>] {
    self.view.aliases()
  }

  pub(crate) fn attributes(&self) -> &[Attribute] {
    self
      .attributes
      .get_or_init(|| self.document().attributes())
      .as_slice()
  }

  pub(crate) fn bindings(&self, document: &Document) -> Arc<Bindings> {
    self.view.bindings(document)
  }

  pub(crate) fn called_functions(&self) -> &HashSet<String> {
    self.usage().called_functions()
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
      .local_declarations(self.recipes())
      .filter_map(|recipe| {
        let attribute = recipe.find_attribute(left)?;

        recipe.find_attribute(right)?;

        Some((recipe, attribute))
      })
      .collect()
  }

  pub(super) fn conflicting_settings(
    &self,
    incompatible: impl Fn(&Setting, &Setting) -> bool,
  ) -> Vec<(&Located<Setting>, &Located<Setting>)> {
    let mut settings = self
      .settings()
      .iter()
      .map(|setting| {
        let groups = GroupSet::from_attributes(&setting.attributes);

        (setting, groups)
      })
      .collect::<Vec<_>>();

    settings.sort_by_key(|(setting, _)| setting.uri == self.document().uri);

    let mut conflicts = Vec::new();

    for (index, (current, current_groups)) in settings.iter().enumerate() {
      if current.uri != self.document().uri {
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

  pub(crate) fn declares_module(&self, path: &Namepath) -> bool {
    match self.view.module(path) {
      ModuleLookup::Found(view) => view
        .modules()
        .iter()
        .any(|module| module.name.value == path.name().value),
      ModuleLookup::Undeclared(_) | ModuleLookup::Unloaded => false,
    }
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
      if declaration.uri != self.document().uri {
        conflicts.record(name(declaration), attributes(declaration));
      }
    }

    self
      .local_declarations(declarations)
      .filter(|declaration| {
        conflicts.record(name(declaration), attributes(declaration))
      })
      .collect()
  }

  pub(crate) fn filesystem(&self) -> &'a dyn Filesystem {
    self.view.filesystem()
  }

  pub(crate) fn find_function(&self, name: &str) -> Option<&Located<Function>> {
    self.view.find_function(name)
  }

  pub(crate) fn function_calls(&self) -> &[FunctionCall] {
    self
      .function_calls
      .get_or_init(|| self.document().function_calls())
      .as_slice()
  }

  pub(crate) fn functions(&self) -> &[Located<Function>] {
    self.view.functions()
  }

  pub(crate) fn loads(&self, path: &Path) -> bool {
    self.view.loads(path)
  }

  pub(super) fn local_declarations<'b, T>(
    &self,
    declarations: &'b [Located<T>],
  ) -> impl Iterator<Item = &'b T> {
    declarations
      .iter()
      .filter(|declaration| declaration.uri == self.document().uri)
      .map(Deref::deref)
  }

  pub(crate) fn modules(&self) -> &[Located<ModuleDeclaration>] {
    self.view.modules()
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

  pub(crate) fn recipes(&self) -> &[Located<Recipe>] {
    self.view.recipes()
  }

  pub(crate) fn resolve_recipe(
    &self,
    path: &Namepath,
  ) -> Option<Located<Recipe>> {
    self.view.resolve_recipe(path)
  }

  pub(crate) fn resolved_functions(
    &self,
  ) -> impl Iterator<Item = &Located<Function>> {
    self.view.resolved_functions()
  }

  pub(crate) fn resolved_recipes(
    &self,
  ) -> impl Iterator<Item = &Located<Recipe>> {
    self.view.resolved_recipes()
  }

  pub(crate) fn resolved_variables(
    &self,
  ) -> impl Iterator<Item = &Located<Variable>> {
    self.view.resolved_variables()
  }

  pub(crate) fn setting_enabled(&self, name: &str) -> bool {
    self.view.setting_enabled(name)
  }

  pub(crate) fn settings(&self) -> &[Located<Setting>] {
    self.view.settings()
  }

  pub(crate) fn tree(&self) -> &Tree {
    &self.document().tree
  }

  pub(crate) fn unexports(&self) -> &[Located<Unexport>] {
    self.view.unexports()
  }

  pub(crate) fn unresolved_identifiers(&self) -> &[(TextNode, Option<String>)] {
    self.usage().unresolved_identifiers(&self.document().uri)
  }

  pub(crate) fn unresolved_recipe(
    &self,
    path: &Namepath,
  ) -> Option<UnresolvedRecipe> {
    let module = match self.view.module(path) {
      ModuleLookup::Found(module) => module,
      ModuleLookup::Undeclared(segment) => {
        return Some(UnresolvedRecipe::Module(segment));
      }
      ModuleLookup::Unloaded => return None,
    };

    let name = path.name();

    if module.find_recipe(&name.value).is_some() {
      return None;
    }

    let suggestion = name
      .value
      .find_suggestion(
        module
          .recipes()
          .iter()
          .map(|recipe| recipe.name.value.as_str()),
      )
      .map(|suggestion| Namepath {
        range: path.range,
        segments: path
          .modules()
          .iter()
          .cloned()
          .chain([TextNode {
            range: name.range,
            value: suggestion,
          }])
          .collect(),
      });

    Some(UnresolvedRecipe::Recipe { suggestion })
  }

  fn usage(&self) -> &Usage {
    self.usage.get_or_init(|| self.view.usage())
  }

  pub(crate) fn used_parameters(&self) -> &[Parameter] {
    self.usage().used_parameters(&self.document().uri)
  }

  pub(crate) fn used_variables(&self) -> &HashSet<String> {
    self.usage().used_variables()
  }

  pub(crate) fn variables(&self) -> &[Located<Variable>] {
    self.view.variables()
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
        .local_declarations(context.recipes())
        .collect::<Vec<_>>(),
      [&document.recipes()[0]],
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
        .recipes()
        .iter()
        .map(|recipe| recipe.name.value.as_str())
        .collect::<Vec<_>>();

      assert_eq!(recipe_names, ["foo", "bar", "baz"]);
    });
  }
}
