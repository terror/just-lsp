use super::*;

pub(crate) struct UnresolvedIdentifier {
  pub(crate) name: String,
  pub(crate) range: lsp::Range,
}

#[derive(Default)]
pub(crate) struct IdentifierAnalysis {
  recipe_identifier_usage: HashMap<String, HashSet<String>>,
  unresolved_identifiers: Vec<UnresolvedIdentifier>,
  variable_usage: HashMap<String, bool>,
}

impl IdentifierAnalysis {
  fn new(context: &RuleContext<'_>) -> Self {
    let mut variable_usage = context
      .variables()
      .iter()
      .map(|variable| (variable.name.value.clone(), false))
      .collect::<HashMap<_, _>>();

    let mut recipe_identifier_usage = context
      .recipes()
      .iter()
      .map(|recipe| (recipe.name.value.clone(), HashSet::new()))
      .collect::<HashMap<_, _>>();

    let mut unresolved_identifiers = Vec::new();

    if let Some(tree) = context.tree() {
      let root = tree.root_node();

      for identifier in root.find_all("expression > value > identifier") {
        Self::record_identifier(
          context,
          &mut recipe_identifier_usage,
          &mut variable_usage,
          &mut unresolved_identifiers,
          identifier,
        );
      }

      for identifier in root.find_all(
        "parameter > value > identifier, variadic_parameter > value > identifier",
      ) {
        Self::record_identifier(
          context,
          &mut recipe_identifier_usage,
          &mut variable_usage,
          &mut unresolved_identifiers,
          identifier,
        );
      }
    }

    Self {
      recipe_identifier_usage,
      unresolved_identifiers,
      variable_usage,
    }
  }

  fn record_identifier(
    context: &RuleContext<'_>,
    recipe_identifier_usage: &mut HashMap<String, HashSet<String>>,
    variable_usage: &mut HashMap<String, bool>,
    unresolved_identifiers: &mut Vec<UnresolvedIdentifier>,
    identifier: Node<'_>,
  ) {
    let document = context.document();
    let recipe_parameters = context.recipe_parameters();
    let value_names = context.variable_and_builtin_names();

    let recipe_name = identifier
      .get_parent("recipe")
      .as_ref()
      .and_then(|recipe_node| recipe_node.find("recipe_header > identifier"))
      .map_or_else(String::new, |identifier_node| {
        document.get_node_text(&identifier_node)
      });

    let identifier_name = document.get_node_text(&identifier);

    if let Some(recipe) = context.recipe(&recipe_name) {
      recipe_identifier_usage
        .entry(recipe.name.value.clone())
        .or_default()
        .insert(identifier_name.clone());

      let containing_parameter = identifier
        .get_parent("parameter")
        .or_else(|| identifier.get_parent("variadic_parameter"));

      if let Some(containing_parameter) = &containing_parameter {
        let containing_parameter_name = document
          .get_node_text(&containing_parameter.find("identifier").unwrap());

        if recipe_parameters
          .get(&recipe.name.value)
          .is_some_and(|parameters| {
            parameters
              .iter()
              .take_while(|parameter| {
                parameter.name != containing_parameter_name
              })
              .any(|parameter| parameter.name == identifier_name)
          })
        {
          return;
        }
      } else if recipe_parameters.get(&recipe.name.value).is_some_and(
        |parameters| {
          parameters
            .iter()
            .any(|parameter| parameter.name == identifier_name)
        },
      ) {
        return;
      }
    }

    if let Some(usage) = variable_usage.get_mut(&identifier_name) {
      *usage = true;
    }

    if !value_names.contains(&identifier_name) {
      unresolved_identifiers.push(UnresolvedIdentifier {
        name: identifier_name,
        range: identifier.get_range(document),
      });
    }
  }
}

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
  identifier_analysis: OnceLock<IdentifierAnalysis>,
  imported_documents: Vec<Document>,
  recipe_names: OnceLock<HashSet<String>>,
  recipe_parameters: OnceLock<HashMap<String, Vec<Parameter>>>,
  recipes: OnceLock<Vec<Recipe>>,
  settings: OnceLock<Vec<Setting>>,
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

  fn identifier_analysis(&self) -> &IdentifierAnalysis {
    self
      .identifier_analysis
      .get_or_init(|| IdentifierAnalysis::new(self))
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
      identifier_analysis: OnceLock::new(),
      imported_documents: Self::resolve_imports(document),
      recipe_names: OnceLock::new(),
      recipe_parameters: OnceLock::new(),
      recipes: OnceLock::new(),
      settings: OnceLock::new(),
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

  pub(crate) fn recipe_identifier_usage(
    &self,
  ) -> &HashMap<String, HashSet<String>> {
    &self.identifier_analysis().recipe_identifier_usage
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

  pub(crate) fn unresolved_identifiers(&self) -> &[UnresolvedIdentifier] {
    &self.identifier_analysis().unresolved_identifiers
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

  pub(crate) fn variable_usage(&self) -> &HashMap<String, bool> {
    &self.identifier_analysis().variable_usage
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
