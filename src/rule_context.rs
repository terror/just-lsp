use super::*;

pub(crate) struct RuleContext<'a> {
  aliases: OnceLock<Vec<Alias>>,
  attributes: OnceLock<Vec<Attribute>>,
  builtin_attributes_map:
    OnceLock<HashMap<&'static str, Vec<&'static Builtin<'static>>>>,
  builtin_function_map:
    OnceLock<HashMap<&'static str, &'static Builtin<'static>>>,
  builtin_setting_map:
    OnceLock<HashMap<&'static str, &'static Builtin<'static>>>,
  document: &'a Document,
  document_variable_names: OnceLock<HashSet<String>>,
  function_calls: OnceLock<Vec<FunctionCall>>,
  functions: OnceLock<Vec<Function>>,
  imported_documents: Vec<Document>,
  recipe_names: OnceLock<HashSet<String>>,
  recipe_parameters: OnceLock<HashMap<String, Vec<Parameter>>>,
  recipes: OnceLock<Vec<Recipe>>,
  scope: OnceLock<Scope>,
  settings: OnceLock<Vec<Setting>>,
  user_function_names: OnceLock<HashSet<String>>,
  variable_and_builtin_names: OnceLock<HashSet<String>>,
  variables: OnceLock<Vec<Variable>>,
}

impl<'a> RuleContext<'a> {
  pub(crate) fn aliases(&self) -> &[Alias] {
    self
      .aliases
      .get_or_init(|| {
        once(self.document)
          .chain(&self.imported_documents)
          .flat_map(Document::aliases)
          .collect()
      })
      .as_slice()
  }

  pub(crate) fn attributes(&self) -> &[Attribute] {
    self
      .attributes
      .get_or_init(|| self.document.attributes())
      .as_slice()
  }

  pub(crate) fn builtin_attributes(
    &self,
    name: &str,
  ) -> &[&'static Builtin<'static>] {
    self
      .builtin_attributes_map()
      .get(name)
      .map_or(&[], Vec::as_slice)
  }

  fn builtin_attributes_map(
    &self,
  ) -> &HashMap<&'static str, Vec<&'static Builtin<'static>>> {
    self.builtin_attributes_map.get_or_init(|| {
      let mut map = HashMap::new();

      for builtin in &BUILTINS {
        if let Builtin::Attribute { name, .. } = builtin {
          map.entry(*name).or_insert_with(Vec::new).push(builtin);
        }
      }

      map
    })
  }

  pub(crate) fn builtin_function(
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

      for builtin in &BUILTINS {
        if let Builtin::Function { name, .. } = builtin {
          map.entry(*name).or_insert(builtin);
        }
      }

      map
    })
  }

  pub(crate) fn builtin_setting(
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

      for builtin in &BUILTINS {
        if let Builtin::Setting { name, .. } = builtin {
          map.entry(*name).or_insert(builtin);
        }
      }

      map
    })
  }

  pub(crate) fn document(&self) -> &Document {
    self.document
  }

  pub(crate) fn document_variable_names(&self) -> &HashSet<String> {
    self.document_variable_names.get_or_init(|| {
      self
        .variables()
        .iter()
        .map(|variable| variable.name.value.clone())
        .collect()
    })
  }

  pub(crate) fn function_calls(&self) -> &[FunctionCall] {
    self
      .function_calls
      .get_or_init(|| self.document.function_calls())
      .as_slice()
  }

  pub(crate) fn functions(&self) -> &[Function] {
    self
      .functions
      .get_or_init(|| {
        once(self.document)
          .chain(&self.imported_documents)
          .flat_map(Document::functions)
          .collect()
      })
      .as_slice()
  }

  pub(crate) fn new(document: &'a Document) -> Self {
    Self {
      aliases: OnceLock::new(),
      attributes: OnceLock::new(),
      builtin_attributes_map: OnceLock::new(),
      builtin_function_map: OnceLock::new(),
      builtin_setting_map: OnceLock::new(),
      document,
      document_variable_names: OnceLock::new(),
      function_calls: OnceLock::new(),
      functions: OnceLock::new(),
      imported_documents: Self::resolve_imports(document),
      recipe_names: OnceLock::new(),
      recipe_parameters: OnceLock::new(),
      recipes: OnceLock::new(),
      scope: OnceLock::new(),
      settings: OnceLock::new(),
      user_function_names: OnceLock::new(),
      variable_and_builtin_names: OnceLock::new(),
      variables: OnceLock::new(),
    }
  }

  pub(crate) fn recipe(&self, name: &str) -> Option<&Recipe> {
    self
      .recipes()
      .iter()
      .find(|recipe| recipe.name.value == name)
  }

  pub(crate) fn recipe_names(&self) -> &HashSet<String> {
    self.recipe_names.get_or_init(|| {
      self
        .recipes()
        .iter()
        .map(|recipe| recipe.name.value.clone())
        .collect()
    })
  }

  pub(crate) fn recipe_parameters(&self) -> &HashMap<String, Vec<Parameter>> {
    self.recipe_parameters.get_or_init(|| {
      self
        .recipes()
        .iter()
        .map(|recipe| (recipe.name.value.clone(), recipe.parameters.clone()))
        .collect()
    })
  }

  pub(crate) fn recipes(&self) -> &[Recipe] {
    self
      .recipes
      .get_or_init(|| {
        once(self.document)
          .chain(&self.imported_documents)
          .flat_map(Document::recipes)
          .collect()
      })
      .as_slice()
  }

  fn resolve_imports(document: &Document) -> Vec<Document> {
    let mut documents = Vec::new();
    let mut seen = HashSet::new();

    if let Ok(path) = document.uri.to_file_path() {
      seen.insert(path);
    }

    Self::resolve_imports_recursive(document, &mut documents, &mut seen);

    documents
  }

  fn resolve_imports_recursive(
    document: &Document,
    documents: &mut Vec<Document>,
    seen: &mut HashSet<PathBuf>,
  ) {
    for import in document.imports() {
      let Some(path) = import.resolve(&document.uri) else {
        continue;
      };

      if !seen.insert(path.clone()) {
        continue;
      }

      let Ok(content) = fs::read_to_string(&path) else {
        if !import.optional {
          log::warn!("failed to read import: {}", path.display());
        }

        continue;
      };

      let Ok(uri) = lsp::Url::from_file_path(&path) else {
        continue;
      };

      let mut imported = Document {
        content: Rope::from_str(&content),
        tree: None,
        uri,
        version: 0,
      };

      if imported.parse().is_err() {
        continue;
      }

      Self::resolve_imports_recursive(&imported, documents, seen);

      documents.push(imported);
    }
  }

  pub(crate) fn scope(&self) -> &Scope {
    self.scope.get_or_init(|| Scope::analyze(self))
  }

  pub(crate) fn setting_enabled(&self, name: &str) -> bool {
    self.settings().iter().any(|setting| {
      setting.name == name && matches!(setting.kind, SettingKind::Boolean(true))
    })
  }

  pub(crate) fn settings(&self) -> &[Setting] {
    self
      .settings
      .get_or_init(|| {
        once(self.document)
          .chain(&self.imported_documents)
          .flat_map(Document::settings)
          .collect()
      })
      .as_slice()
  }

  pub(crate) fn tree(&self) -> Option<&Tree> {
    self.document.tree.as_ref()
  }

  pub(crate) fn user_function_names(&self) -> &HashSet<String> {
    self.user_function_names.get_or_init(|| {
      self
        .functions()
        .iter()
        .map(|function| function.name.value.clone())
        .collect()
    })
  }

  pub(crate) fn variable_and_builtin_names(&self) -> &HashSet<String> {
    self.variable_and_builtin_names.get_or_init(|| {
      let mut names = self.document_variable_names().clone();

      names.extend(BUILTINS.iter().filter_map(|builtin| match builtin {
        Builtin::Constant { name, .. } => Some((*name).to_owned()),
        _ => None,
      }));

      names
    })
  }

  pub(crate) fn variables(&self) -> &[Variable] {
    self
      .variables
      .get_or_init(|| {
        once(self.document)
          .chain(&self.imported_documents)
          .flat_map(Document::variables)
          .collect()
      })
      .as_slice()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

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

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str(
        &fs::read_to_string(dir.path().join("justfile")).unwrap(),
      ),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let context = RuleContext::new(&document);

    let recipe_names = context
      .recipes()
      .iter()
      .map(|recipe| recipe.name.value.as_str())
      .collect::<Vec<_>>();

    assert_eq!(recipe_names, ["foo", "bar"]);
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

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str(
        &fs::read_to_string(dir.path().join("justfile")).unwrap(),
      ),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let context = RuleContext::new(&document);

    let variable_names = context
      .variables()
      .iter()
      .map(|variable| variable.name.value.as_str())
      .collect::<Vec<_>>();

    assert_eq!(variable_names, ["foo", "bar"]);
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

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str(
        &fs::read_to_string(dir.path().join("justfile")).unwrap(),
      ),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let context = RuleContext::new(&document);

    let setting_names = context
      .settings()
      .iter()
      .map(|s| s.name.as_str())
      .collect::<Vec<_>>();

    assert_eq!(setting_names, ["dotenv-load", "export"]);
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

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str(
        &fs::read_to_string(dir.path().join("justfile")).unwrap(),
      ),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let context = RuleContext::new(&document);

    let recipe_names = context
      .recipes()
      .iter()
      .map(|recipe| recipe.name.value.as_str())
      .collect::<Vec<_>>();

    assert_eq!(recipe_names, ["foo"]);
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

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str(
        &fs::read_to_string(dir.path().join("justfile")).unwrap(),
      ),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let context = RuleContext::new(&document);

    let recipe_names = context
      .recipes()
      .iter()
      .map(|recipe| recipe.name.value.as_str())
      .collect::<Vec<_>>();

    assert_eq!(recipe_names, ["foo", "baz", "bar"]);
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

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str(
        &fs::read_to_string(dir.path().join("justfile")).unwrap(),
      ),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let context = RuleContext::new(&document);

    let recipe_names = context
      .recipes()
      .iter()
      .map(|recipe| recipe.name.value.as_str())
      .collect::<Vec<_>>();

    assert_eq!(recipe_names, ["foo", "bar"]);
  }
}
