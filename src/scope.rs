use super::*;

#[derive(Default)]
struct LocalScope {
  parameters: HashSet<String>,
  recipe: Option<String>,
}

pub struct Scope<'a> {
  document: &'a Document,
  globals: HashSet<String>,
  pub recipe_identifier_usage: HashMap<String, HashSet<String>>,
  root: bool,
  pub unresolved_identifiers: Vec<(TextNode, Option<String>)>,
  pub variable_usage: HashMap<String, bool>,
}

impl<'a> Scope<'a> {
  pub fn analyze(context: &RuleContext<'a>) -> Self {
    let mut scope = Self::new(context);

    scope.walk_document(context.document(), true);

    for document in context.imported_documents() {
      scope.walk_document(document, false);
    }

    scope
  }

  fn new(context: &RuleContext<'a>) -> Self {
    Self {
      document: context.document(),
      globals: context.variable_and_builtin_names().clone(),
      recipe_identifier_usage: context
        .recipes()
        .iter()
        .map(|recipe| (recipe.name.value.clone(), HashSet::new()))
        .collect(),
      root: true,
      unresolved_identifiers: Vec::new(),
      variable_usage: context
        .variables()
        .iter()
        .map(|variable| (variable.name.value.clone(), false))
        .collect(),
    }
  }

  /// Resolve an identifier against the scope stack.
  ///
  /// Recipe identifier usage is recorded unconditionally before resolution, so
  /// parameter self-references like `foo foo` still count as usage for the
  /// `unused-recipe-parameters` rule. Unresolved identifiers are only recorded from
  /// the analyzed document: usage in imported documents still counts, since
  /// `just` imports are textual inclusions, but a range from an imported
  /// document would not be valid in the analyzed document.
  fn record(&mut self, identifier: Node<'_>, local: &LocalScope) {
    if identifier.is_missing() {
      return;
    }

    let name = self.document.get_node_text(&identifier);

    if let Some(recipe_name) = &local.recipe {
      self
        .recipe_identifier_usage
        .entry(recipe_name.clone())
        .or_default()
        .insert(name.clone());
    }

    if local.parameters.contains(&name) {
      return;
    }

    if let Some(used) = self.variable_usage.get_mut(&name) {
      *used = true;
      return;
    }

    if self.globals.contains(&name) {
      return;
    }

    if self.root {
      let suggestion = name.find_suggestion(
        local
          .parameters
          .iter()
          .chain(&self.globals)
          .map(String::as_str),
      );

      self.unresolved_identifiers.push((
        TextNode {
          value: name,
          range: identifier.get_range(self.document),
        },
        suggestion,
      ));
    }
  }

  /// Walk one document's tree, recording the usage of its recipes, functions,
  /// and top-level expressions.
  fn walk_document(&mut self, document: &'a Document, root: bool) {
    self.document = document;
    self.root = root;

    let root_node = document.tree.root_node();

    for node in root_node.find_all("recipe") {
      self.walk_recipe(node);
    }

    for node in root_node.find_all("function_definition") {
      self.walk_function(node);
    }

    let local = LocalScope::default();

    for identifier in root_node.find_all("value > identifier") {
      if identifier.has_any_parent(&["function_definition", "recipe"]) {
        continue;
      }

      self.record(identifier, &local);
    }
  }

  /// Enter a function definition scope and record its body.
  ///
  /// Parameters are all defined before processing the body, since `just`
  /// function parameters have no default values and cannot reference each
  /// other.
  fn walk_function(&mut self, function_node: Node<'_>) {
    let local = LocalScope {
      parameters: function_node
        .child_by_field_name("parameters")
        .iter()
        .flat_map(|parameters| parameters.find_all("^identifier"))
        .map(|parameter| self.document.get_node_text(&parameter))
        .collect(),
      recipe: None,
    };

    if let Some(body_node) = function_node.child_by_field_name("body") {
      for identifier in body_node.find_all("value > identifier") {
        self.record(identifier, &local);
      }
    }
  }

  /// Enter a recipe scope and record its parameters and body.
  ///
  /// Parameters are defined one at a time: each default value is recorded
  /// before defining that parameter, so `b=a` resolves `a` against earlier
  /// parameters and globals but not `b` itself. Body identifiers inside
  /// parameter defaults are skipped in the final expression walk to avoid
  /// double-recording with the wrong scope.
  fn walk_recipe(&mut self, recipe_node: Node<'_>) {
    let Some(name_node) = recipe_node.find("recipe_header > identifier") else {
      return;
    };

    let mut local = LocalScope {
      parameters: HashSet::new(),
      recipe: Some(self.document.get_node_text(&name_node)),
    };

    if let Some(parameters_node) =
      recipe_node.find("recipe_header > parameters")
    {
      for parameter_node in
        parameters_node.find_all("^parameter, ^variadic_parameter")
      {
        let parameter_node = if parameter_node.kind() == "variadic_parameter" {
          parameter_node.find("parameter")
        } else {
          Some(parameter_node)
        };

        let Some(parameter_node) = parameter_node else {
          continue;
        };

        if let Some(default_node) =
          parameter_node.child_by_field_name("default")
        {
          for identifier in default_node.find_all("value > identifier") {
            self.record(identifier, &local);
          }
        }

        if let Some(name_node) = parameter_node.child_by_field_name("name") {
          local
            .parameters
            .insert(self.document.get_node_text(&name_node));
        }
      }
    }

    for identifier in recipe_node.find_all("value > identifier") {
      if identifier.has_any_parent(&["parameter", "variadic_parameter"]) {
        continue;
      }

      self.record(identifier, &local);
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  struct Test {
    document: Document,
    imported_documents: Vec<Document>,
    recipe_usage: Vec<(&'static str, Vec<&'static str>)>,
    unresolved: Vec<&'static str>,
    unused: Vec<&'static str>,
    used: Vec<&'static str>,
  }

  impl Test {
    fn imported_document(self, content: &str) -> Self {
      Self {
        imported_documents: self
          .imported_documents
          .into_iter()
          .chain([Document::from(content)])
          .collect(),
        ..self
      }
    }

    fn new(content: &str) -> Self {
      Self {
        document: Document::from(content),
        imported_documents: Vec::new(),
        recipe_usage: Vec::new(),
        unresolved: Vec::new(),
        unused: Vec::new(),
        used: Vec::new(),
      }
    }

    fn recipe_usage(
      self,
      recipe: &'static str,
      names: &[&'static str],
    ) -> Self {
      Self {
        recipe_usage: self
          .recipe_usage
          .into_iter()
          .chain(once((recipe, names.to_vec())))
          .collect(),
        ..self
      }
    }

    fn run(self) {
      let view = ProjectView {
        document: &self.document,
        documents: once(&self.document)
          .chain(&self.imported_documents)
          .enumerate()
          .map(|(index, document)| ProjectViewDocument {
            document,
            load_depth: usize::from(index > 0),
          })
          .collect(),
      };

      let scope = Scope::analyze(&RuleContext::new(&view));

      let mut actual_unresolved = scope
        .unresolved_identifiers
        .iter()
        .map(|(identifier, _)| identifier.value.as_str())
        .collect::<Vec<_>>();

      let mut expected_unresolved = self.unresolved.clone();

      actual_unresolved.sort_unstable();

      expected_unresolved.sort_unstable();

      assert_eq!(
        actual_unresolved, expected_unresolved,
        "unresolved mismatch"
      );

      let mut actual_used = scope
        .variable_usage
        .iter()
        .filter(|(_, used)| **used)
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();

      actual_used.sort_unstable();

      let mut expected_used = self.used.clone();

      expected_used.sort_unstable();

      assert_eq!(actual_used, expected_used, "used variables mismatch");

      let mut actual_unused = scope
        .variable_usage
        .iter()
        .filter(|(_, used)| !**used)
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();

      actual_unused.sort_unstable();

      let mut expected_unused = self.unused.clone();

      expected_unused.sort_unstable();

      assert_eq!(actual_unused, expected_unused, "unused variables mismatch");

      for (recipe, expected_names) in &self.recipe_usage {
        let mut actual_names = scope
          .recipe_identifier_usage
          .get(*recipe)
          .map(|set| set.iter().map(String::as_str).collect::<Vec<_>>())
          .unwrap_or_default();

        actual_names.sort_unstable();

        let mut expected = expected_names.clone();

        expected.sort_unstable();

        assert_eq!(actual_names, expected, "recipe `{recipe}` usage mismatch");
      }
    }

    fn unresolved(self, names: &[&'static str]) -> Self {
      Self {
        unresolved: names.to_vec(),
        ..self
      }
    }

    fn unused(self, names: &[&'static str]) -> Self {
      Self {
        unused: names.to_vec(),
        ..self
      }
    }

    fn used(self, names: &[&'static str]) -> Self {
      Self {
        used: names.to_vec(),
        ..self
      }
    }
  }

  #[test]
  fn builtin_constants_resolve() {
    Test::new(indoc! {
      "
      foo:
        echo {{HEX}}
      "
    })
    .run();
  }

  #[test]
  fn complex_parameter_ordering() {
    Test::new(indoc! {
      "
      foo a b=a c=b:
        echo {{c}}
      "
    })
    .run();
  }

  #[test]
  fn empty_justfile() {
    Test::new("").run();
  }

  #[test]
  fn function_body_references_variable() {
    Test::new(indoc! {
      "
      set unstable

      base := 'foo'

      join(ext) := base + '.' + ext
      "
    })
    .used(&["base"])
    .run();
  }

  #[test]
  fn function_body_undefined_identifier() {
    Test::new(indoc! {
      "
      set unstable

      join(ext) := missing + '.' + ext
      "
    })
    .unresolved(&["missing"])
    .run();
  }

  #[test]
  fn function_parameter_does_not_leak() {
    Test::new(indoc! {
      "
      set unstable

      add(a) := a + 'x'

      foo:
        echo {{a}}
      "
    })
    .unresolved(&["a"])
    .run();
  }

  #[test]
  fn function_parameter_resolves_in_body() {
    Test::new(indoc! {
      "
      set unstable

      add(a) := a + 'x'
      "
    })
    .run();
  }

  #[test]
  fn imported_document_assignment_references_variable() {
    Test::new(indoc! {
      "
      lib_var := 'x'
      "
    })
    .imported_document(indoc! {
      "
      helper := lib_var
      "
    })
    .used(&["lib_var"])
    .unused(&["helper"])
    .run();
  }

  #[test]
  fn imported_document_function_body_references_variable() {
    Test::new(indoc! {
      "
      set unstable

      lib_var := 'x'
      "
    })
    .imported_document(indoc! {
      "
      join(ext) := lib_var + ext
      "
    })
    .used(&["lib_var"])
    .run();
  }

  #[test]
  fn imported_document_recipe_body_references_variable() {
    Test::new(indoc! {
      "
      import 'lib.just'

      lib_var := 'x'

      foo: use
      "
    })
    .imported_document(indoc! {
      "
      use:
        echo {{lib_var}}
      "
    })
    .recipe_usage("use", &["lib_var"])
    .used(&["lib_var"])
    .run();
  }

  #[test]
  fn imported_document_recipe_parameter_default_references_variable() {
    Test::new(indoc! {
      "
      lib_var := 'x'
      "
    })
    .imported_document(indoc! {
      "
      use arg=lib_var:
        echo {{arg}}
      "
    })
    .recipe_usage("use", &["arg", "lib_var"])
    .used(&["lib_var"])
    .run();
  }

  #[test]
  fn imported_document_recipe_parameter_usage() {
    Test::new(indoc! {
      "
      import 'lib.just'

      foo: use
      "
    })
    .imported_document(indoc! {
      "
      use arg:
        echo {{arg}}
      "
    })
    .recipe_usage("use", &["arg"])
    .run();
  }

  #[test]
  fn multiple_function_parameters() {
    Test::new(indoc! {
      "
      set unstable

      add(a, b) := a + b
      "
    })
    .run();
  }

  #[test]
  fn multiple_parameters_in_recipe() {
    Test::new(indoc! {
      "
      foo a b c:
        echo {{a}} {{b}} {{c}}
      "
    })
    .recipe_usage("foo", &["a", "b", "c"])
    .run();
  }

  #[test]
  fn multiple_recipes_isolated_scopes() {
    Test::new(indoc! {
      "
      foo a:
        echo {{a}}

      bar b:
        echo {{b}}
      "
    })
    .recipe_usage("foo", &["a"])
    .recipe_usage("bar", &["b"])
    .run();
  }

  #[test]
  fn multiple_unresolved_identifiers() {
    Test::new(indoc! {
      "
      foo:
        echo {{a}} {{b}} {{c}}
      "
    })
    .unresolved(&["a", "b", "c"])
    .run();
  }

  #[test]
  fn multiple_variables_usage_tracking() {
    Test::new(indoc! {
      "
      a := 'foo'
      b := 'bar'
      c := 'baz'

      recipe:
        echo {{a}} {{c}}
      "
    })
    .used(&["a", "c"])
    .unused(&["b"])
    .run();
  }

  #[test]
  fn parameter_default_cannot_reference_itself() {
    Test::new(indoc! {
      "
      foo a=a:
        echo {{a}}
      "
    })
    .unresolved(&["a"])
    .run();
  }

  #[test]
  fn parameter_default_cannot_reference_later_parameter() {
    Test::new(indoc! {
      "
      foo a=b b='x':
        echo {{a}}
      "
    })
    .unresolved(&["b"])
    .run();
  }

  #[test]
  fn parameter_default_references_earlier_parameter() {
    Test::new(indoc! {
      "
      foo a b=a:
        echo {{b}}
      "
    })
    .run();
  }

  #[test]
  fn parameter_default_references_variable() {
    Test::new(indoc! {
      "
      x := 'foo'

      bar y=x:
        echo {{y}}
      "
    })
    .used(&["x"])
    .run();
  }

  #[test]
  fn parameter_shadows_variable_in_recipe() {
    Test::new(indoc! {
      "
      x := 'foo'

      bar x:
        echo {{x}}
      "
    })
    .unused(&["x"])
    .run();
  }

  #[test]
  fn parameters_do_not_shadow_globals_in_assignments() {
    #[track_caller]
    fn case(definition: &str) {
      let content = format!("{definition}\nbaz := bar\n");

      Test::new(&format!("bar := 'foo'\n{content}"))
        .recipe_usage("foo", &[])
        .used(&["bar"])
        .unused(&["baz"])
        .run();

      Test::new("bar := 'foo'\n")
        .imported_document(&content)
        .recipe_usage("foo", &[])
        .used(&["bar"])
        .unused(&["baz"])
        .run();
    }

    case("foo bar:");
    case("foo(bar) := bar");
  }

  #[test]
  fn recipe_identifier_usage_parameter_default_self_reference() {
    Test::new(indoc! {
      "
      x := 'bar'

      foo a=a:
        echo {{a}}
      "
    })
    .unresolved(&["a"])
    .recipe_usage("foo", &["a"])
    .unused(&["x"])
    .run();
  }

  #[test]
  fn recipe_identifier_usage_tracks_body() {
    Test::new(indoc! {
      "
      x := 'foo'

      bar:
        echo {{x}}
      "
    })
    .used(&["x"])
    .recipe_usage("bar", &["x"])
    .run();
  }

  #[test]
  fn recipe_identifier_usage_tracks_parameters() {
    Test::new(indoc! {
      "
      foo bar:
        echo {{bar}}
      "
    })
    .recipe_usage("foo", &["bar"])
    .run();
  }

  #[test]
  fn recipe_parameter_does_not_leak_to_other_recipes() {
    Test::new(indoc! {
      "
      foo bar:
        echo {{bar}}

      baz:
        echo {{bar}}
      "
    })
    .unresolved(&["bar"])
    .run();
  }

  #[test]
  fn recipe_parameter_resolves_in_body() {
    Test::new(indoc! {
      "
      foo bar:
        echo {{bar}}
      "
    })
    .run();
  }

  #[test]
  fn recipe_parameters_do_not_leak_across_documents() {
    Test::new(indoc! {
      "
      import 'lib.just'

      foo param:
        echo {{param}}
      "
    })
    .imported_document(indoc! {
      "
      param := 'x'

      helper := param
      "
    })
    .used(&["param"])
    .unused(&["helper"])
    .run();
  }

  #[test]
  fn recipe_with_no_parameters_or_body() {
    Test::new(indoc! {
      "
      foo:
      "
    })
    .recipe_usage("foo", &[])
    .run();
  }

  #[test]
  fn undefined_identifier_in_assignment() {
    Test::new(indoc! {
      "
      foo := bar
      "
    })
    .unresolved(&["bar"])
    .unused(&["foo"])
    .run();
  }

  #[test]
  fn undefined_identifier_in_recipe() {
    Test::new(indoc! {
      "
      foo:
        echo {{bar}}
      "
    })
    .unresolved(&["bar"])
    .run();
  }

  #[test]
  fn unresolved_identifiers_not_recorded_from_imports() {
    Test::new(indoc! {
      "
      foo:
        echo {{missing}}
      "
    })
    .imported_document(indoc! {
      "
      use:
        echo {{absent}}
      "
    })
    .unresolved(&["missing"])
    .run();
  }

  #[test]
  fn user_defined_function_resolves() {
    Test::new(indoc! {
      "
      set unstable

      greet(name) := f\"hello {name}\"

      foo:
        echo {{greet('world')}}
      "
    })
    .run();
  }

  #[test]
  fn variable_chain() {
    Test::new(indoc! {
      "
      a := 'foo'
      b := a
      c := b
      "
    })
    .used(&["a", "b"])
    .unused(&["c"])
    .run();
  }

  #[test]
  fn variable_defined_and_unused() {
    Test::new(indoc! {
      "
      foo := 'bar'
      "
    })
    .unused(&["foo"])
    .run();
  }

  #[test]
  fn variable_used_across_multiple_recipes() {
    Test::new(indoc! {
      "
      x := 'foo'

      a:
        echo {{x}}

      b:
        echo {{x}}
      "
    })
    .used(&["x"])
    .recipe_usage("a", &["x"])
    .recipe_usage("b", &["x"])
    .run();
  }

  #[test]
  fn variable_used_in_assignment() {
    Test::new(indoc! {
      "
      foo := 'bar'
      baz := foo
      "
    })
    .used(&["foo"])
    .unused(&["baz"])
    .run();
  }

  #[test]
  fn variable_used_in_parameter_default_and_body() {
    Test::new(indoc! {
      "
      x := 'foo'

      bar y=x:
        echo {{x}} {{y}}
      "
    })
    .used(&["x"])
    .run();
  }

  #[test]
  fn variable_used_in_recipe_body() {
    Test::new(indoc! {
      "
      foo := 'bar'

      baz:
        echo {{foo}}
      "
    })
    .used(&["foo"])
    .run();
  }

  #[test]
  fn variable_used_in_unary_assignment() {
    Test::new(indoc! {
      "
      foo := 'bar'
      baz := !foo
      "
    })
    .used(&["foo"])
    .unused(&["baz"])
    .run();
  }

  #[test]
  fn variadic_parameter_resolves_in_body() {
    Test::new(indoc! {
      "
      foo +bar:
        echo {{bar}}
      "
    })
    .run();
  }

  #[test]
  fn variadic_parameter_with_default() {
    Test::new(indoc! {
      "
      x := 'foo'

      bar +args=x:
        echo {{args}}
      "
    })
    .used(&["x"])
    .run();
  }

  #[test]
  fn variadic_star_parameter_resolves_in_body() {
    Test::new(indoc! {
      "
      foo *bar:
        echo {{bar}}
      "
    })
    .run();
  }
}
