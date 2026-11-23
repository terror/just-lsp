use super::*;

pub(crate) struct Scope {
  pub(crate) recipe_identifier_usage: HashMap<String, HashSet<String>>,
  pub(crate) unresolved_identifiers: Vec<(TextNode, Option<String>)>,
  pub(crate) variable_usage: HashMap<String, bool>,
}

impl Scope {
  pub(crate) fn analyze(context: &RuleContext<'_>) -> Self {
    let view = context.view();

    let root = Bindings::new(view, context.document());

    let imported = context
      .imported_documents()
      .map(|document| Bindings::new(view, document))
      .collect::<Vec<_>>();

    let mut variable_usage = context
      .variables()
      .iter()
      .map(|variable| (variable.name.value.clone(), false))
      .collect::<HashMap<_, _>>();

    for bound in once(&root)
      .chain(&imported)
      .flat_map(Bindings::identifiers)
      .filter(|bound| bound.expression)
    {
      if let Binding::Variable(name) = &bound.binding
        && let Some(used) = variable_usage.get_mut(name)
      {
        *used = true;
      }
    }

    let unresolved_identifiers = root
      .identifiers()
      .iter()
      .filter_map(|bound| match &bound.binding {
        Binding::Unresolved { suggestion } => {
          Some((bound.identifier.clone(), suggestion.clone()))
        }
        _ => None,
      })
      .collect();

    Self {
      recipe_identifier_usage: root.recipe_identifiers,
      unresolved_identifiers,
      variable_usage,
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  struct Test {
    project: TestProject,
    recipe_usage: Vec<(&'static str, Vec<&'static str>)>,
    unresolved: Vec<&'static str>,
    unused: Vec<&'static str>,
    used: Vec<&'static str>,
  }

  impl Test {
    fn imported_document(mut self, content: &str) -> Self {
      self.project.imported_document(content);
      self
    }

    fn new(content: &str) -> Self {
      Self {
        project: TestProject::new(Document::from(content)),
        recipe_usage: Vec::new(),
        unresolved: Vec::new(),
        unused: Vec::new(),
        used: Vec::new(),
      }
    }

    fn recipe_usage(
      mut self,
      recipe: &'static str,
      names: &[&'static str],
    ) -> Self {
      self.recipe_usage.push((recipe, names.to_vec()));
      self
    }

    #[track_caller]
    fn run(self) {
      let view = self.project.view();

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
          .map_or_default(|set| {
            set.iter().map(String::as_str).collect::<Vec<_>>()
          });

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
    .recipe_usage("use", &[])
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
    .recipe_usage("use", &[])
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
    .recipe_usage("use", &[])
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
