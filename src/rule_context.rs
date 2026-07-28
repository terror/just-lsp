use super::*;

type BuiltinRef = &'static Builtin<'static>;

struct Items<T> {
  groups: Vec<GroupSet>,
  values: Vec<T>,
}

impl<T> Items<T> {
  fn iter(&self) -> impl Iterator<Item = (&T, &GroupSet)> {
    self.values.iter().zip(&self.groups)
  }
}

pub struct RuleContext<'a> {
  aliases: OnceLock<Items<Alias>>,
  attributes: OnceLock<Vec<Attribute>>,
  builtin_attributes_map: OnceLock<HashMap<&'static str, Vec<BuiltinRef>>>,
  builtin_function_map: OnceLock<HashMap<&'static str, BuiltinRef>>,
  builtin_setting_map: OnceLock<HashMap<&'static str, BuiltinRef>>,
  document: &'a Document,
  document_variable_names: OnceLock<HashSet<String>>,
  function_calls: OnceLock<Vec<FunctionCall>>,
  functions: OnceLock<Items<Function>>,
  imported_documents: Vec<ImportedDocument>,
  recipe_names: OnceLock<HashSet<String>>,
  recipe_parameters: OnceLock<HashMap<String, Vec<Parameter>>>,
  recipes: OnceLock<Items<Recipe>>,
  scope: OnceLock<Scope<'a>>,
  settings: OnceLock<Items<Setting>>,
  unexports: OnceLock<Items<Unexport>>,
  user_function_names: OnceLock<HashSet<String>>,
  variable_and_builtin_names: OnceLock<HashSet<String>>,
  variables: OnceLock<Items<Variable>>,
}

impl<'a> RuleContext<'a> {
  fn alias_items(&self) -> &Items<Alias> {
    self.aliases.get_or_init(|| {
      self.collect_items(Document::aliases, |alias| &alias.attributes)
    })
  }

  pub fn aliases(&self) -> &[Alias] {
    &self.alias_items().values
  }

  pub fn aliases_with_groups(
    &self,
  ) -> impl Iterator<Item = (&Alias, &GroupSet)> {
    self.alias_items().iter()
  }

  pub fn attributes(&self) -> &[Attribute] {
    self
      .attributes
      .get_or_init(|| self.document.attributes())
      .as_slice()
  }

  pub fn builtin_attributes(&self, name: &str) -> &[&'static Builtin<'static>] {
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

      for builtin in BUILTINS {
        if let Builtin::Attribute { name, .. } = builtin {
          map.entry(*name).or_insert_with(Vec::new).push(builtin);
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

  fn collect_items<T>(
    &self,
    extract: impl Fn(&Document) -> Vec<T>,
    attributes: impl Fn(&T) -> &[Attribute],
  ) -> Items<T> {
    let mut groups = Vec::new();
    let mut values = Vec::new();

    for (document, inherited) in self.documents() {
      for value in extract(document) {
        let item_groups = GroupSet::from_attributes(attributes(&value));
        let item_groups = inherited.map_or_else(
          || item_groups.clone(),
          |inherited| inherited.intersection(&item_groups),
        );

        if !item_groups.is_empty() {
          groups.push(item_groups);
          values.push(value);
        }
      }
    }

    Items { groups, values }
  }

  pub fn document(&self) -> &'a Document {
    self.document
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

  fn documents(&self) -> impl Iterator<Item = (&Document, Option<&GroupSet>)> {
    once((self.document, None)).chain(
      self
        .imported_documents
        .iter()
        .map(|imported| (&imported.document, Some(&imported.groups))),
    )
  }

  pub fn function_calls(&self) -> &[FunctionCall] {
    self
      .function_calls
      .get_or_init(|| self.document.function_calls())
      .as_slice()
  }

  fn function_items(&self) -> &Items<Function> {
    self.functions.get_or_init(|| {
      self.collect_items(Document::functions, |function| &function.attributes)
    })
  }

  pub fn functions(&self) -> &[Function] {
    &self.function_items().values
  }

  pub fn functions_with_groups(
    &self,
  ) -> impl Iterator<Item = (&Function, &GroupSet)> {
    self.function_items().iter()
  }

  fn lexical_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
      match component {
        Component::CurDir => {}
        Component::ParentDir => {
          if !normalized.pop() && !normalized.has_root() {
            normalized.push(component);
          }
        }
        _ => normalized.push(component),
      }
    }

    normalized
  }

  #[must_use]
  pub fn new(document: &'a Document) -> Self {
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
      unexports: OnceLock::new(),
      user_function_names: OnceLock::new(),
      variable_and_builtin_names: OnceLock::new(),
      variables: OnceLock::new(),
    }
  }

  pub fn recipe(&self, name: &str) -> Option<&Recipe> {
    self.recipe_with_groups(name).map(|(recipe, _)| recipe)
  }

  fn recipe_items(&self) -> &Items<Recipe> {
    self.recipes.get_or_init(|| {
      self.collect_items(Document::recipes, |recipe| &recipe.attributes)
    })
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

  pub fn recipe_parameters(&self) -> &HashMap<String, Vec<Parameter>> {
    self.recipe_parameters.get_or_init(|| {
      self
        .recipes()
        .iter()
        .map(|recipe| (recipe.name.value.clone(), recipe.parameters.clone()))
        .collect()
    })
  }

  pub fn recipe_with_groups(&self, name: &str) -> Option<(&Recipe, &GroupSet)> {
    self
      .recipes_with_groups()
      .find(|(recipe, _)| recipe.name.value == name)
  }

  pub fn recipes(&self) -> &[Recipe] {
    &self.recipe_items().values
  }

  pub fn recipes_with_groups(
    &self,
  ) -> impl Iterator<Item = (&Recipe, &GroupSet)> {
    self.recipe_items().iter()
  }

  fn resolve_imports(document: &Document) -> Vec<ImportedDocument> {
    let mut documents = Vec::new();
    let mut active = HashSet::new();
    let mut expanded = HashMap::new();

    let groups = GroupSet::from([Group::Any]);

    if let Ok(path) = document.uri.to_file_path()
      && let Ok(path) = path.canonicalize()
    {
      active.insert(path);
    }

    Self::resolve_imports_recursive(
      document,
      &groups,
      &mut documents,
      &mut active,
      &mut expanded,
    );

    documents
  }

  fn resolve_imports_recursive(
    document: &Document,
    inherited: &GroupSet,
    documents: &mut Vec<ImportedDocument>,
    active: &mut HashSet<PathBuf>,
    expanded: &mut HashMap<PathBuf, GroupSet>,
  ) {
    for import in document.imports() {
      let groups =
        inherited.intersection(&GroupSet::from_attributes(&import.attributes));

      if groups.is_empty() {
        continue;
      }

      let Some(path) = import.resolve(&document.uri) else {
        continue;
      };

      let path = Self::lexical_path(&path);

      let Ok(identity) = path.canonicalize() else {
        if !import.optional {
          warn!(path = %path.display(), "failed to read import");
        }

        continue;
      };

      if active.contains(&identity)
        || expanded
          .get(&path)
          .is_some_and(|previous| previous.covers(&groups))
      {
        continue;
      }

      let Ok(content) = fs::read_to_string(&path) else {
        if !import.optional {
          warn!(path = %path.display(), "failed to read import");
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

      expanded.entry(path).or_default().union_with(groups.clone());

      let document_index = documents.iter().position(|candidate| {
        candidate
          .document
          .uri
          .to_file_path()
          .ok()
          .and_then(|path| path.canonicalize().ok())
          .is_some_and(|path| path == identity)
      });

      if let Some(index) = document_index {
        documents[index].groups.union_with(groups.clone());
      }

      active.insert(identity.clone());

      Self::resolve_imports_recursive(
        &imported, &groups, documents, active, expanded,
      );

      active.remove(&identity);

      if document_index.is_none() {
        documents.push(ImportedDocument {
          document: imported,
          groups,
        });
      }
    }
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

  pub fn setting_enabled_for(&self, name: &str, groups: &GroupSet) -> bool {
    let mut enabled = GroupSet::default();

    for (setting, setting_groups) in self.settings_with_groups() {
      if setting.name.value == name
        && matches!(setting.kind, SettingKind::Boolean(true))
      {
        enabled.union_with(setting_groups.clone());
      }
    }

    enabled.covers(groups)
  }

  pub fn setting_enabled_in(&self, name: &str, groups: &GroupSet) -> bool {
    self
      .settings_with_groups()
      .any(|(setting, setting_groups)| {
        setting.name.value == name
          && matches!(setting.kind, SettingKind::Boolean(true))
          && setting_groups.conflicts_with(groups)
      })
  }

  fn setting_items(&self) -> &Items<Setting> {
    self.settings.get_or_init(|| {
      self.collect_items(Document::settings, |setting| &setting.attributes)
    })
  }

  pub fn settings(&self) -> &[Setting] {
    &self.setting_items().values
  }

  pub fn settings_with_groups(
    &self,
  ) -> impl Iterator<Item = (&Setting, &GroupSet)> {
    self.setting_items().iter()
  }

  pub fn tree(&self) -> Option<&Tree> {
    self.document.tree.as_ref()
  }

  fn unexport_items(&self) -> &Items<Unexport> {
    self.unexports.get_or_init(|| {
      self.collect_items(Document::unexports, |unexport| &unexport.attributes)
    })
  }

  pub fn unexports(&self) -> &[Unexport] {
    &self.unexport_items().values
  }

  pub fn unexports_with_groups(
    &self,
  ) -> impl Iterator<Item = (&Unexport, &GroupSet)> {
    self.unexport_items().iter()
  }

  pub fn user_function_names(&self) -> &HashSet<String> {
    self.user_function_names.get_or_init(|| {
      self
        .functions()
        .iter()
        .map(|function| function.name.value.clone())
        .collect()
    })
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

  fn variable_items(&self) -> &Items<Variable> {
    self.variables.get_or_init(|| {
      self.collect_items(Document::variables, |variable| &variable.attributes)
    })
  }

  pub fn variables(&self) -> &[Variable] {
    &self.variable_items().values
  }

  pub fn variables_with_groups(
    &self,
  ) -> impl Iterator<Item = (&Variable, &GroupSet)> {
    self.variable_items().iter()
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*, indoc::indoc, pretty_assertions::assert_eq, tempfile::Builder,
  };

  #[test]
  fn imported_recipes_are_merged() {
    let dir = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::write(
      dir.path().join("bar.just"),
      indoc! {
        "
        [linux]
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

    assert_eq!(
      context.recipes()[1].attributes,
      vec![Attribute {
        arguments: Vec::new(),
        name: TextNode {
          range: lsp::Range::at(0, 1, 0, 6),
          value: "linux".into(),
        },
        range: lsp::Range::at(0, 0, 1, 0),
        target: Some(AttributeTarget::Recipe),
      }]
    );
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
        [linux]
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
      .map(|s| s.name.value.as_str())
      .collect::<Vec<_>>();

    assert_eq!(setting_names, ["dotenv-load", "export"]);
    assert!(
      context.setting_enabled_for("export", &GroupSet::from([Group::Linux]))
    );
    assert!(
      !context.setting_enabled_for("export", &GroupSet::from([Group::Windows]))
    );
    assert!(context.setting_enabled("export"));
    assert!(
      !context.setting_enabled_for("export", &GroupSet::from([Group::Any]))
    );
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
  fn platform_gated_imports_are_merged() {
    let dir = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::write(dir.path().join("bar.just"), "bar:\n  echo bar\n").unwrap();

    fs::write(
      dir.path().join("justfile"),
      indoc! {
        "
        [linux]
        import 'bar.just'
        [windows]
        import 'bar.just'
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

    assert_eq!(
      context.recipes_with_groups().next().unwrap().1,
      &GroupSet::from([Group::Linux, Group::Windows])
    );
  }

  #[test]
  fn platform_gated_imports_intersect() {
    let dir = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::write(dir.path().join("baz.just"), "baz:\n  echo baz\n").unwrap();

    fs::write(
      dir.path().join("bar.just"),
      indoc! {
        "
        [linux]
        import 'baz.just'
        "
      },
    )
    .unwrap();

    fs::write(
      dir.path().join("justfile"),
      indoc! {
        "
        [unix]
        import 'bar.just'
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

    assert_eq!(
      context.recipes_with_groups().next().unwrap().1,
      &GroupSet::from([Group::Linux])
    );
    assert!(context.recipes()[0].attributes.is_empty());
  }

  #[test]
  fn unconditional_import_overrides_platform_gate() {
    let dir = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::write(dir.path().join("bar.just"), "bar:\n  echo bar\n").unwrap();

    fs::write(
      dir.path().join("justfile"),
      indoc! {
        "
        [linux]
        import 'bar.just'
        import 'bar.just'
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

    assert_eq!(
      context.recipes_with_groups().next().unwrap().1,
      &GroupSet::from([Group::Any])
    );
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

  #[cfg(unix)]
  #[test]
  fn relative_imports_from_symlinks_use_lexical_parent() {
    let dir = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::create_dir(dir.path().join("alias")).unwrap();
    fs::create_dir(dir.path().join("shared")).unwrap();

    fs::write(
      dir.path().join("shared/bar.just"),
      indoc! {
        "
        import 'baz.just'

        bar:
          echo bar
        "
      },
    )
    .unwrap();

    fs::write(dir.path().join("alias/baz.just"), "baz:\n  echo baz\n").unwrap();

    std::os::unix::fs::symlink(
      dir.path().join("shared/bar.just"),
      dir.path().join("alias/bar.just"),
    )
    .unwrap();

    fs::write(
      dir.path().join("justfile"),
      "import 'alias/bar.just'\n\nfoo:\n  echo foo\n",
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

  #[cfg(unix)]
  #[test]
  fn symlink_import_contexts_are_expanded_independently() {
    let dir = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::create_dir(dir.path().join("one")).unwrap();
    fs::create_dir(dir.path().join("shared")).unwrap();
    fs::create_dir(dir.path().join("two")).unwrap();

    fs::write(
      dir.path().join("shared/bar.just"),
      "import 'baz.just'\n\nbar:\n  echo bar\n",
    )
    .unwrap();

    fs::write(dir.path().join("one/baz.just"), "one:\n  echo one\n").unwrap();

    fs::write(dir.path().join("two/baz.just"), "two:\n  echo two\n").unwrap();

    for directory in ["one", "two"] {
      std::os::unix::fs::symlink(
        dir.path().join("shared/bar.just"),
        dir.path().join(directory).join("bar.just"),
      )
      .unwrap();
    }

    fs::write(
      dir.path().join("justfile"),
      "import 'one/bar.just'\nimport 'two/bar.just'\n\nfoo:\n  echo foo\n",
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

    assert_eq!(recipe_names, ["foo", "one", "bar", "two"]);
  }

  #[test]
  fn circular_imports_are_handled() {
    let dir = Builder::new().prefix("just-lsp").tempdir().unwrap();

    fs::create_dir(dir.path().join("sub")).unwrap();

    fs::write(
      dir.path().join("sub/bar.just"),
      indoc! {
        "
        import '../justfile'

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
        import 'sub/bar.just'

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
