use super::*;

pub struct Scope<'a> {
  current_recipe: Option<String>,
  document: &'a Document,
  globals: HashSet<String>,
  locals: HashSet<String>,
  pub recipe_identifier_usage: HashMap<String, HashSet<String>>,
  pub unresolved_identifiers: Vec<(String, lsp::Range)>,
  pub variable_usage: HashMap<String, bool>,
}

impl<'a> Scope<'a> {
  pub fn analyze(context: &RuleContext<'a>) -> Self {
    let mut scope = Self::new(context);

    let Some(tree) = context.tree() else {
      return scope;
    };

    let root = tree.root_node();

    for node in root.find_all("recipe") {
      scope.walk_recipe(node);
    }

    for node in root.find_all("function_definition") {
      scope.walk_function(node);
    }

    for identifier in root.find_all("expression > value > identifier") {
      if identifier.has_any_parent(&["function_definition", "recipe"]) {
        continue;
      }

      scope.record(identifier);
    }

    scope
  }

  fn new(context: &RuleContext<'a>) -> Self {
    Self {
      current_recipe: None,
      document: context.document(),
      globals: context
        .variable_and_builtin_names()
        .iter()
        .cloned()
        .chain(context.user_function_names().iter().cloned())
        .collect(),
      locals: HashSet::new(),
      recipe_identifier_usage: context
        .recipes()
        .iter()
        .map(|recipe| (recipe.name.value.clone(), HashSet::new()))
        .collect(),
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
  /// Recipe identifier usage is recorded unconditionally before resolution,
  /// so parameter self-references like `foo foo` still count as usage for the
  /// `unused-parameters` rule.
  fn record(&mut self, identifier: Node<'_>) {
    if identifier.start_byte() == identifier.end_byte() {
      return;
    }

    let name = self.document.get_node_text(&identifier);

    if let Some(recipe_name) = &self.current_recipe {
      self
        .recipe_identifier_usage
        .entry(recipe_name.clone())
        .or_default()
        .insert(name.clone());
    }

    if self.locals.contains(&name) {
      return;
    }

    if let Some(used) = self.variable_usage.get_mut(&name) {
      *used = true;
      return;
    }

    if self.globals.contains(&name) {
      return;
    }

    self
      .unresolved_identifiers
      .push((name, identifier.get_range(self.document)));
  }

  /// Enter a function definition scope and record its body.
  ///
  /// Parameters are all defined before processing the body, since `just`
  /// function parameters have no default values and cannot reference each
  /// other.
  fn walk_function(&mut self, function_node: Node<'_>) {
    self.locals.clear();

    if let Some(parameters_node) =
      function_node.child_by_field_name("parameters")
    {
      for parameter_node in parameters_node.find_all("^identifier") {
        self
          .locals
          .insert(self.document.get_node_text(&parameter_node));
      }
    }

    if let Some(body_node) = function_node.child_by_field_name("body") {
      for identifier in body_node.find_all("value > identifier") {
        self.record(identifier);
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

    self.current_recipe = Some(self.document.get_node_text(&name_node));
    self.locals.clear();

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
          for identifier in default_node
            .find_all("^identifier, expression > value > identifier")
          {
            self.record(identifier);
          }
        }

        if let Some(name_node) = parameter_node.child_by_field_name("name") {
          self.locals.insert(self.document.get_node_text(&name_node));
        }
      }
    }

    for identifier in recipe_node.find_all("expression > value > identifier") {
      if identifier.has_any_parent(&["parameter", "variadic_parameter"]) {
        continue;
      }

      self.record(identifier);
    }

    self.current_recipe = None;
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  struct Test {
    document: Document,
    recipe_usage: Vec<(&'static str, Vec<&'static str>)>,
    unresolved: Vec<&'static str>,
    unused: Vec<&'static str>,
    used: Vec<&'static str>,
  }

  impl Test {
    fn new(content: &str) -> Self {
      Self {
        document: Document::from(content),
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
      let scope = Scope::analyze(&RuleContext::new(&self.document));

      let mut actual_unresolved = scope
        .unresolved_identifiers
        .iter()
        .map(|(name, _)| name.as_str())
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
  fn empty_justfile() {
    Test::new("").run();
  }

  #[test]
  fn variable_defined_and_unused() {
    Test::new("foo := 'bar'\n").unused(&["foo"]).run();
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
  fn undefined_identifier_in_assignment() {
    Test::new("foo := bar\n")
      .unresolved(&["bar"])
      .unused(&["foo"])
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
  fn variadic_star_parameter_resolves_in_body() {
    Test::new(indoc! {
      "
      foo *bar:
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
}
