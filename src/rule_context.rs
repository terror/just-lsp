use super::*;

type BuiltinRef = &'static Builtin<'static>;

pub struct RuleContext<'a> {
  aliases: OnceLock<Vec<Located<Alias>>>,
  attributes: OnceLock<Vec<Attribute>>,
  builtin_attribute_map: OnceLock<HashMap<&'static str, BuiltinRef>>,
  builtin_function_map: OnceLock<HashMap<&'static str, BuiltinRef>>,
  builtin_setting_map: OnceLock<HashMap<&'static str, BuiltinRef>>,
  document_variable_names: OnceLock<HashSet<String>>,
  function_calls: OnceLock<Vec<FunctionCall>>,
  functions: OnceLock<Vec<Located<Function>>>,
  recipe_names: OnceLock<HashSet<String>>,
  recipes: OnceLock<Vec<Located<Recipe>>>,
  scope: OnceLock<Scope<'a>>,
  settings: OnceLock<Vec<Located<Setting>>>,
  unexports: OnceLock<Vec<Located<Unexport>>>,
  user_function_names: OnceLock<HashSet<String>>,
  variable_and_builtin_names: OnceLock<HashSet<String>>,
  variables: OnceLock<Vec<Located<Variable>>>,
  view: &'a ProjectView<'a>,
}

impl<'a> RuleContext<'a> {
  pub fn aliases(&self) -> &[Located<Alias>] {
    self
      .aliases
      .get_or_init(|| self.declarations(Document::aliases))
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

  pub(super) fn conflicting_settings(
    &self,
    incompatible: impl Fn(&Setting, &Setting) -> bool,
  ) -> Vec<Diagnostic> {
    let mut settings = self
      .settings()
      .iter()
      .map(|setting| {
        let groups = GroupSet::from_attributes(&setting.attributes);

        (setting, groups)
      })
      .collect::<Vec<_>>();

    settings.sort_by_key(|(setting, _)| setting.uri == self.document().uri);

    let mut diagnostics = Vec::new();

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
          diagnostics.push(Diagnostic::error(
            format!(
              "`{}` is incompatible with `{}`",
              previous.name.value, current.name.value,
            ),
            current.range,
          ));
        }
      }
    }

    diagnostics
  }

  fn declarations<T>(
    &self,
    declarations: impl Fn(&Document) -> Vec<T>,
  ) -> Vec<Located<T>> {
    self
      .view
      .documents()
      .flat_map(|document| {
        declarations(document)
          .into_iter()
          .map(|declaration| Located::new(document.uri.clone(), declaration))
      })
      .collect()
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

  pub fn function_calls(&self) -> &[FunctionCall] {
    self
      .function_calls
      .get_or_init(|| self.document().function_calls())
      .as_slice()
  }

  pub fn functions(&self) -> &[Located<Function>] {
    self
      .functions
      .get_or_init(|| self.declarations(Document::functions))
      .as_slice()
  }

  pub fn imported_documents(&self) -> impl Iterator<Item = &'a Document> + '_ {
    self
      .view
      .documents()
      .filter(|document| document.uri != self.document().uri)
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

  pub fn recipes(&self) -> &[Located<Recipe>] {
    self
      .recipes
      .get_or_init(|| self.declarations(Document::recipes))
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

  pub fn settings(&self) -> &[Located<Setting>] {
    self
      .settings
      .get_or_init(|| self.declarations(Document::settings))
      .as_slice()
  }

  pub fn tree(&self) -> &Tree {
    &self.document().tree
  }

  pub fn unexports(&self) -> &[Located<Unexport>] {
    self
      .unexports
      .get_or_init(|| self.declarations(Document::unexports))
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

  pub fn variables(&self) -> &[Located<Variable>] {
    self
      .variables
      .get_or_init(|| self.declarations(Document::variables))
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
  fn analyzed_document_is_selected_by_uri() {
    let root =
      Document::new("foo:\n", lsp::Url::parse("file:///foo.just").unwrap())
        .unwrap();

    let document =
      Document::new("foo:\n", lsp::Url::parse("file:///bar.just").unwrap())
        .unwrap();

    let view = ProjectView {
      document: &document,
      documents: vec![
        ProjectViewDocument {
          document: &root,
          load_depth: 0,
        },
        ProjectViewDocument {
          document: &document,
          load_depth: 1,
        },
      ],
    };

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

    assert_eq!(
      context
        .imported_documents()
        .map(|document| &document.uri)
        .collect::<Vec<_>>(),
      [&root.uri],
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
