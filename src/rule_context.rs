use super::*;

pub(crate) struct RuleContext<'a> {
  attributes: Vec<Attribute>,
  bindings: Arc<Bindings>,
  function_calls: Vec<FunctionCall>,
  usage: Arc<Usage>,
  view: &'a ModuleView<'a>,
}

impl<'a> RuleContext<'a> {
  pub(crate) fn aliases(
    &self,
    scope: Scope,
  ) -> impl Iterator<Item = &'a Located<Alias>> {
    self.scoped(scope, self.view.aliases())
  }

  pub(crate) fn attributes(&self) -> &[Attribute] {
    &self.attributes
  }

  pub(crate) fn bindings(&self, document: &Document) -> Arc<Bindings> {
    self.view.bindings(document)
  }

  pub(crate) fn called_functions(&self) -> &HashSet<String> {
    &self.usage.called_functions
  }

  pub(crate) fn circular_dependencies(&self) -> &'a [ProjectDependency] {
    self.view.circular_dependencies()
  }

  pub(super) fn conflicting_recipe_attributes(
    &self,
    left: &str,
    right: &str,
  ) -> Vec<(&'a Located<Recipe>, &'a Attribute)> {
    self
      .recipes(Scope::Local)
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
    let settings = |scope| {
      self
        .settings(scope)
        .map(|setting| {
          (setting, GroupSet::from_attributes(&setting.attributes))
        })
        .collect::<Vec<_>>()
    };

    let (imported, local) = (settings(Scope::Imported), settings(Scope::Local));

    let mut conflicts = Vec::new();

    for (index, (current, current_groups)) in local.iter().enumerate() {
      let mut seen = HashSet::new();

      for (previous, previous_groups) in imported.iter().chain(&local[..index])
      {
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
  ) -> impl Iterator<Item = &'a ProjectDependency> {
    self.view.dependencies()
  }

  pub(crate) fn document(&self) -> &'a Document {
    self.view.document()
  }

  pub(crate) fn documents(&self) -> impl Iterator<Item = &'a Document> {
    self.view.documents()
  }

  pub(super) fn duplicate_declarations<T: 'a>(
    &self,
    declarations: impl IntoIterator<Item = &'a Located<T>>,
    name: impl Fn(&T) -> &TextNode,
    attributes: impl Fn(&T) -> &[Attribute],
  ) -> Vec<&'a T> {
    let (local, imported) = declarations
      .into_iter()
      .partition::<Vec<_>, _>(|declaration| self.is_local(declaration));

    let mut conflicts = ConflictTracker::default();

    for declaration in imported {
      conflicts.record(name(declaration), attributes(declaration));
    }

    local
      .into_iter()
      .map(Deref::deref)
      .filter(|declaration| {
        conflicts.record(name(declaration), attributes(declaration))
      })
      .collect()
  }

  pub(crate) fn filesystem(&self) -> &'a dyn Filesystem {
    self.view.filesystem()
  }

  pub(crate) fn find_function(
    &self,
    name: &str,
  ) -> Option<&'a Located<Function>> {
    self.view.find_function(name)
  }

  pub(crate) fn function_calls(&self) -> &[FunctionCall] {
    &self.function_calls
  }

  pub(crate) fn functions(
    &self,
    scope: Scope,
  ) -> impl Iterator<Item = &'a Located<Function>> {
    self.scoped(scope, self.view.functions())
  }

  fn is_local<T>(&self, declaration: &Located<T>) -> bool {
    Scope::Local.contains(&self.document().uri, declaration)
  }

  pub(crate) fn loads(&self, path: &Path) -> bool {
    self.view.loads(path)
  }

  pub(crate) fn modules(
    &self,
    scope: Scope,
  ) -> impl Iterator<Item = &'a Located<ModuleDeclaration>> {
    self.scoped(scope, self.view.modules())
  }

  #[must_use]
  pub(crate) fn new(view: &'a ModuleView<'a>) -> Self {
    let document = view.document();

    Self {
      attributes: document.attributes(),
      bindings: view.bindings(document),
      function_calls: document.function_calls(),
      usage: view.usage(),
      view,
    }
  }

  pub(crate) fn recipes(
    &self,
    scope: Scope,
  ) -> impl Iterator<Item = &'a Located<Recipe>> {
    self.scoped(scope, self.view.recipes())
  }

  pub(crate) fn resolve_recipe(
    &self,
    path: &Namepath,
  ) -> Option<Located<Recipe>> {
    self.view.resolve_recipe(path)
  }

  pub(crate) fn resolved_functions(
    &self,
    scope: Scope,
  ) -> impl Iterator<Item = &'a Located<Function>> {
    self.scoped(scope, self.view.resolved_functions())
  }

  pub(crate) fn resolved_recipes(
    &self,
    scope: Scope,
  ) -> impl Iterator<Item = &'a Located<Recipe>> {
    self.scoped(scope, self.view.resolved_recipes())
  }

  pub(crate) fn resolved_variables(
    &self,
    scope: Scope,
  ) -> impl Iterator<Item = &'a Located<Variable>> {
    self.scoped(scope, self.view.resolved_variables())
  }

  fn scoped<T: 'a>(
    &self,
    scope: Scope,
    declarations: impl IntoIterator<Item = &'a Located<T>>,
  ) -> impl Iterator<Item = &'a Located<T>> {
    let document = &self.document().uri;

    declarations
      .into_iter()
      .filter(move |declaration| scope.contains(document, declaration))
  }

  pub(crate) fn setting_enabled(&self, name: &str) -> bool {
    self.view.setting_enabled(name)
  }

  pub(crate) fn settings(
    &self,
    scope: Scope,
  ) -> impl Iterator<Item = &'a Located<Setting>> {
    self.scoped(scope, self.view.settings())
  }

  pub(crate) fn tree(&self) -> &'a Tree {
    &self.document().tree
  }

  pub(crate) fn unexports(
    &self,
    scope: Scope,
  ) -> impl Iterator<Item = &'a Located<Unexport>> {
    self.scoped(scope, self.view.unexports())
  }

  pub(crate) fn unresolved_identifiers(
    &self,
  ) -> impl Iterator<Item = (&TextNode, &Option<String>)> {
    self.bindings.unresolved_identifiers()
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

  pub(crate) fn used_parameters(&self) -> impl Iterator<Item = &Parameter> {
    self.bindings.used_parameters()
  }

  pub(crate) fn used_variables(&self) -> &HashSet<String> {
    &self.usage.used_variables
  }

  pub(crate) fn variables(
    &self,
    scope: Scope,
  ) -> impl Iterator<Item = &'a Located<Variable>> {
    self.scoped(scope, self.view.variables())
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
        .recipes(Scope::Module)
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
        .recipes(Scope::Module)
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
        .settings(Scope::Module)
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
        .variables(Scope::Module)
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
        .recipes(Scope::Module)
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
        .recipes(Scope::Module)
        .map(|recipe| recipe.name.value.as_str())
        .collect::<Vec<_>>();

      assert_eq!(recipe_names, ["foo", "bar", "baz"]);
    });
  }

  #[test]
  fn scope_filters_declarations_by_document() {
    #[track_caller]
    fn case(context: &RuleContext, scope: Scope, expected: &[&lsp::Url]) {
      assert_eq!(
        context
          .recipes(scope)
          .map(|recipe| &recipe.uri)
          .collect::<Vec<_>>(),
        expected,
      );
    }

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

    case(&context, Scope::Imported, &[&root.uri]);
    case(&context, Scope::Local, &[&document.uri]);
    case(&context, Scope::Module, &[&root.uri, &document.uri]);
  }
}
