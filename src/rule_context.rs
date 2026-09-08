use super::*;

type BuiltinRef = &'static Builtin<'static>;

pub struct RuleContext<'a> {
  aliases: OnceLock<Vec<Alias>>,
  attributes: OnceLock<Vec<Attribute>>,
  builtin_attribute_map: OnceLock<HashMap<&'static str, BuiltinRef>>,
  builtin_function_map: OnceLock<HashMap<&'static str, BuiltinRef>>,
  builtin_setting_map: OnceLock<HashMap<&'static str, BuiltinRef>>,
  document_variable_names: OnceLock<HashSet<String>>,
  function_calls: OnceLock<Vec<FunctionCall>>,
  functions: OnceLock<Vec<Function>>,
  recipe_names: OnceLock<HashSet<String>>,
  recipes: OnceLock<Vec<Recipe>>,
  scope: OnceLock<Scope<'a>>,
  settings: OnceLock<Vec<Setting>>,
  unexports: OnceLock<Vec<Unexport>>,
  user_function_names: OnceLock<HashSet<String>>,
  variable_and_builtin_names: OnceLock<HashSet<String>>,
  variables: OnceLock<Vec<Variable>>,
  view: &'a ProjectView<'a>,
}

impl<'a> RuleContext<'a> {
  pub fn aliases(&self) -> &[Alias] {
    self
      .aliases
      .get_or_init(|| self.documents().flat_map(Document::aliases).collect())
      .as_slice()
  }

  pub fn attributes(&self) -> &[Attribute] {
    self
      .attributes
      .get_or_init(|| self.document().attributes())
      .as_slice()
  }

  pub fn builtin_attribute(
    &self,
    name: &str,
  ) -> Option<&'static Builtin<'static>> {
    self.builtin_attribute_map().get(name).copied()
  }

  fn builtin_attribute_map(
    &self,
  ) -> &HashMap<&'static str, &'static Builtin<'static>> {
    self.builtin_attribute_map.get_or_init(|| {
      let mut map = HashMap::new();

      for builtin in BUILTINS {
        if let Builtin::Attribute { name, .. } = builtin {
          map.entry(*name).or_insert(builtin);
        }
      }

      map
    })
  }

  pub fn builtin_function(
    &self,
    name: &str,
  ) -> Option<&'static Builtin<'static>> {
    self.builtin_function_map().get(name).copied()
  }

  fn builtin_function_map(
    &self,
  ) -> &HashMap<&'static str, &'static Builtin<'static>> {
    self.builtin_function_map.get_or_init(|| {
      let mut map = HashMap::new();

      for builtin in BUILTINS {
        if let Builtin::Function { name, aliases, .. } = builtin {
          map.entry(*name).or_insert(builtin);

          for alias in *aliases {
            map.entry(*alias).or_insert(builtin);
          }
        }
      }

      map
    })
  }

  pub fn builtin_setting(
    &self,
    name: &str,
  ) -> Option<&'static Builtin<'static>> {
    self.builtin_setting_map().get(name).copied()
  }

  fn builtin_setting_map(
    &self,
  ) -> &HashMap<&'static str, &'static Builtin<'static>> {
    self.builtin_setting_map.get_or_init(|| {
      let mut map = HashMap::new();

      for builtin in BUILTINS {
        if let Builtin::Setting { name, .. } = builtin {
          map.entry(*name).or_insert(builtin);
        }
      }

      map
    })
  }

  pub fn document(&self) -> &'a Document {
    self.view.document()
  }

  pub fn document_variable_names(&self) -> &HashSet<String> {
    self.document_variable_names.get_or_init(|| {
      self
        .variables()
        .iter()
        .map(|variable| variable.name.value.clone())
        .collect()
    })
  }

  fn documents(&self) -> impl Iterator<Item = &Document> {
    self.view.documents()
  }

  pub fn function_calls(&self) -> &[FunctionCall] {
    self
      .function_calls
      .get_or_init(|| self.document().function_calls())
      .as_slice()
  }

  pub fn functions(&self) -> &[Function] {
    self
      .functions
      .get_or_init(|| self.documents().flat_map(Document::functions).collect())
      .as_slice()
  }

  pub fn imported_documents(&self) -> impl Iterator<Item = &'a Document> + '_ {
    self.view.documents().skip(1)
  }

  #[must_use]
  pub fn new(view: &'a ProjectView<'a>) -> Self {
    Self {
      aliases: OnceLock::new(),
      attributes: OnceLock::new(),
      builtin_attribute_map: OnceLock::new(),
      builtin_function_map: OnceLock::new(),
      builtin_setting_map: OnceLock::new(),
      document_variable_names: OnceLock::new(),
      function_calls: OnceLock::new(),
      functions: OnceLock::new(),
      recipe_names: OnceLock::new(),
      recipes: OnceLock::new(),
      scope: OnceLock::new(),
      settings: OnceLock::new(),
      unexports: OnceLock::new(),
      user_function_names: OnceLock::new(),
      variable_and_builtin_names: OnceLock::new(),
      variables: OnceLock::new(),
      view,
    }
  }

  pub fn recipe_names(&self) -> &HashSet<String> {
    self.recipe_names.get_or_init(|| {
      self
        .recipes()
        .iter()
        .map(|recipe| recipe.name.value.clone())
        .collect()
    })
  }

  pub fn recipes(&self) -> &[Recipe] {
    self
      .recipes
      .get_or_init(|| self.documents().flat_map(Document::recipes).collect())
      .as_slice()
  }

  pub fn scope(&self) -> &Scope<'_> {
    self.scope.get_or_init(|| Scope::analyze(self))
  }

  pub fn setting_enabled(&self, name: &str) -> bool {
    self.settings().iter().any(|setting| {
      setting.name.value == name
        && matches!(setting.kind, SettingKind::Boolean(true))
    })
  }

  pub fn settings(&self) -> &[Setting] {
    self
      .settings
      .get_or_init(|| self.documents().flat_map(Document::settings).collect())
      .as_slice()
  }

  pub fn tree(&self) -> Option<&Tree> {
    self.document().tree.as_ref()
  }

  pub fn unexports(&self) -> &[Unexport] {
    self
      .unexports
      .get_or_init(|| self.documents().flat_map(Document::unexports).collect())
      .as_slice()
  }

  pub fn user_function_names(&self) -> &HashSet<String> {
    self
      .user_function_names
      .get_or_init(|| self.view.resolved_functions().into_keys().collect())
  }

  pub fn variable_and_builtin_names(&self) -> &HashSet<String> {
    self.variable_and_builtin_names.get_or_init(|| {
      let mut names = self.document_variable_names().clone();

      names.extend(BUILTINS.iter().filter_map(|builtin| match builtin {
        Builtin::Constant { name, .. } => Some((*name).to_owned()),
        _ => None,
      }));

      names
    })
  }

  pub fn variables(&self) -> &[Variable] {
    self
      .variables
      .get_or_init(|| self.documents().flat_map(Document::variables).collect())
      .as_slice()
  }

  pub fn view(&self) -> &ProjectView<'a> {
    self.view
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

    test(&RuleContext::new(&ProjectView::new(
      documents.get(&uri).unwrap(),
      &project.import_scope,
      &documents,
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
