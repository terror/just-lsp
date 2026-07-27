use super::*;

type BuiltinRef = &'static Builtin<'static>;

pub struct RuleContext<'a> {
  aliases: OnceLock<Vec<Alias>>,
  attributes: OnceLock<Vec<Attribute>>,
  builtin_attributes_map: OnceLock<HashMap<&'static str, Vec<BuiltinRef>>>,
  builtin_function_map: OnceLock<HashMap<&'static str, BuiltinRef>>,
  builtin_setting_map: OnceLock<HashMap<&'static str, BuiltinRef>>,
  document: &'a Document,
  document_variable_names: OnceLock<HashSet<String>>,
  function_calls: OnceLock<Vec<FunctionCall>>,
  functions: OnceLock<Vec<Function>>,
  imported_documents: Vec<ImportedDocument>,
  recipe_names: OnceLock<HashSet<String>>,
  recipe_parameters: OnceLock<HashMap<String, Vec<Parameter>>>,
  recipes: OnceLock<Vec<Recipe>>,
  scope: OnceLock<Scope<'a>>,
  settings: OnceLock<Vec<Setting>>,
  unexports: OnceLock<Vec<Unexport>>,
  user_function_names: OnceLock<HashSet<String>>,
  variable_and_builtin_names: OnceLock<HashSet<String>>,
  variables: OnceLock<Vec<Variable>>,
}

impl<'a> RuleContext<'a> {
  pub fn aliases(&self) -> &[Alias] {
    self
      .aliases
      .get_or_init(|| {
        self
          .documents()
          .flat_map(|(document, groups)| {
            Self::with_inherited_groups(document.aliases(), groups, |alias| {
              &mut alias.attributes
            })
          })
          .collect()
      })
      .as_slice()
  }

  fn apply_inherited_groups(
    attributes: &mut Vec<Attribute>,
    inherited: Option<&GroupSet>,
  ) -> bool {
    let Some(inherited) = inherited else {
      return true;
    };

    let item_groups = GroupSet::from_attributes(attributes);

    let groups = inherited.intersection(&item_groups);

    if groups.is_empty() {
      return false;
    }

    if groups == item_groups {
      return true;
    }

    attributes.retain(|attribute| {
      !GroupSet::is_platform_attribute(&attribute.name.value)
    });

    attributes.extend(groups.platform_attribute_names().map(|name| {
      Attribute {
        arguments: Vec::new(),
        name: TextNode {
          range: lsp::Range::default(),
          value: name.to_owned(),
        },
        range: lsp::Range::default(),
        target: None,
      }
    }));

    true
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

  pub fn functions(&self) -> &[Function] {
    self
      .functions
      .get_or_init(|| {
        self
          .documents()
          .flat_map(|(document, groups)| {
            Self::with_inherited_groups(
              document.functions(),
              groups,
              |function| &mut function.attributes,
            )
          })
          .collect()
      })
      .as_slice()
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
    self
      .recipes()
      .iter()
      .find(|recipe| recipe.name.value == name)
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

  pub fn recipes(&self) -> &[Recipe] {
    self
      .recipes
      .get_or_init(|| {
        self
          .documents()
          .flat_map(|(document, groups)| {
            Self::with_inherited_groups(document.recipes(), groups, |recipe| {
              &mut recipe.attributes
            })
          })
          .collect()
      })
      .as_slice()
  }

  fn resolve_imports(document: &Document) -> Vec<ImportedDocument> {
    let mut documents = Vec::new();
    let mut seen = HashMap::new();

    let groups = GroupSet::from([Group::Any]);

    if let Ok(path) = document.uri.to_file_path() {
      seen.insert(path, groups.clone());
    }

    Self::resolve_imports_recursive(
      document,
      &groups,
      &mut documents,
      &mut seen,
    );

    documents
  }

  fn resolve_imports_recursive(
    document: &Document,
    inherited: &GroupSet,
    documents: &mut Vec<ImportedDocument>,
    seen: &mut HashMap<PathBuf, GroupSet>,
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

      if seen
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

      seen.entry(path).or_default().union_with(groups.clone());

      let document_index = documents
        .iter()
        .position(|candidate| candidate.document.uri == imported.uri);

      if let Some(index) = document_index {
        documents[index].groups.union_with(groups.clone());
      }

      Self::resolve_imports_recursive(&imported, &groups, documents, seen);

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

  pub fn settings(&self) -> &[Setting] {
    self
      .settings
      .get_or_init(|| {
        self
          .documents()
          .flat_map(|(document, groups)| {
            Self::with_inherited_groups(
              document.settings(),
              groups,
              |setting| &mut setting.attributes,
            )
          })
          .collect()
      })
      .as_slice()
  }

  pub fn tree(&self) -> Option<&Tree> {
    self.document.tree.as_ref()
  }

  pub fn unexports(&self) -> &[Unexport] {
    self
      .unexports
      .get_or_init(|| {
        self
          .documents()
          .flat_map(|(document, groups)| {
            Self::with_inherited_groups(
              document.unexports(),
              groups,
              |unexport| &mut unexport.attributes,
            )
          })
          .collect()
      })
      .as_slice()
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

  pub fn variables(&self) -> &[Variable] {
    self
      .variables
      .get_or_init(|| {
        self
          .documents()
          .flat_map(|(document, groups)| {
            Self::with_inherited_groups(
              document.variables(),
              groups,
              |variable| &mut variable.attributes,
            )
          })
          .collect()
      })
      .as_slice()
  }

  fn with_inherited_groups<T>(
    mut items: Vec<T>,
    inherited: Option<&GroupSet>,
    attributes: impl Fn(&mut T) -> &mut Vec<Attribute>,
  ) -> Vec<T> {
    items.retain_mut(|item| {
      Self::apply_inherited_groups(attributes(item), inherited)
    });

    items
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
      context.recipes()[0].groups(),
      GroupSet::from([Group::Linux, Group::Windows])
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
      context.recipes()[0].groups(),
      GroupSet::from([Group::Linux])
    );
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

    assert_eq!(context.recipes()[0].groups(), GroupSet::from([Group::Any]));
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
