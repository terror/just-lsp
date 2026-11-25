use super::*;

#[derive(Debug)]
pub(crate) struct Usage {
  pub(crate) called_functions: HashSet<String>,
  pub(crate) used_variables: HashSet<String>,
}

impl From<&ModuleView<'_>> for Usage {
  fn from(view: &ModuleView<'_>) -> Self {
    let bindings = view
      .documents()
      .map(|document| view.bindings(document))
      .collect::<Vec<_>>();

    Self {
      called_functions: view
        .documents()
        .flat_map(Document::function_calls)
        .map(|function_call| function_call.name.value)
        .collect(),
      used_variables: bindings
        .iter()
        .flat_map(|bindings| bindings.identifiers())
        .filter(|bound| bound.expression)
        .filter_map(|bound| match &bound.binding {
          Binding::Variable(name) => Some(name.clone()),
          _ => None,
        })
        .collect(),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  struct Test {
    files: Vec<(&'static str, String)>,
    unresolved: Vec<&'static str>,
    used: Vec<&'static str>,
    used_parameters: Option<Vec<&'static str>>,
  }

  impl Test {
    fn file(mut self, path: &'static str, content: &str) -> Self {
      self.files.push((path, content.to_owned()));
      self
    }

    fn new(content: &str) -> Self {
      Self {
        files: vec![("justfile", content.to_owned())],
        unresolved: Vec::new(),
        used: Vec::new(),
        used_parameters: None,
      }
    }

    #[track_caller]
    fn run(self) {
      let module = TestModule::new(
        &self
          .files
          .iter()
          .map(|(path, content)| (*path, content.as_str()))
          .collect::<Vec<_>>(),
      );

      let view = module.view();

      let usage = Usage::from(&view);

      let bindings = view.bindings(view.document());

      let mut actual_unresolved = bindings
        .unresolved_identifiers()
        .map(|(identifier, _)| identifier.value.as_str())
        .collect::<Vec<_>>();

      let mut expected_unresolved = self.unresolved.clone();

      actual_unresolved.sort_unstable();

      expected_unresolved.sort_unstable();

      assert_eq!(
        actual_unresolved, expected_unresolved,
        "unresolved mismatch"
      );

      let mut actual_used = usage
        .used_variables
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

      actual_used.sort_unstable();

      let mut expected_used = self.used.clone();

      expected_used.sort_unstable();

      assert_eq!(actual_used, expected_used, "used variables mismatch");

      if let Some(mut expected_used_parameters) = self.used_parameters {
        let mut actual_used_parameters = bindings
          .used_parameters()
          .map(|parameter| parameter.name.as_str())
          .collect::<Vec<_>>();

        actual_used_parameters.sort_unstable();

        expected_used_parameters.sort_unstable();

        assert_eq!(
          actual_used_parameters, expected_used_parameters,
          "used parameters mismatch"
        );
      }
    }

    fn unresolved(self, names: &[&'static str]) -> Self {
      Self {
        unresolved: names.to_vec(),
        ..self
      }
    }

    fn used(self, names: &[&'static str]) -> Self {
      Self {
        used: names.to_vec(),
        ..self
      }
    }

    fn used_parameters(self, names: &[&'static str]) -> Self {
      Self {
        used_parameters: Some(names.to_vec()),
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
      import 'foo.just'
      "
    })
    .file(
      "foo.just",
      indoc! {
        "
        helper := lib_var
        "
      },
    )
    .used(&["lib_var"])
    .run();
  }

  #[test]
  fn imported_document_function_body_references_variable() {
    Test::new(indoc! {
      "
      set unstable

      lib_var := 'x'
      import 'foo.just'
      "
    })
    .file(
      "foo.just",
      indoc! {
        "
        join(ext) := lib_var + ext
        "
      },
    )
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
    .file(
      "lib.just",
      indoc! {
        "
        use:
          echo {{lib_var}}
        "
      },
    )
    .used(&["lib_var"])
    .run();
  }

  #[test]
  fn imported_document_recipe_parameter_default_references_variable() {
    Test::new(indoc! {
      "
      lib_var := 'x'
      import 'foo.just'
      "
    })
    .file(
      "foo.just",
      indoc! {
        "
        use arg=lib_var:
          echo {{arg}}
        "
      },
    )
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
    .file(
      "lib.just",
      indoc! {
        "
        use arg:
          echo {{arg}}
        "
      },
    )
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
    .used_parameters(&["a", "b", "c"])
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
    .used_parameters(&["a", "b"])
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
    .run();
  }

  #[test]
  fn parameter_usage_ignores_variables() {
    Test::new(indoc! {
      "
      x := 'foo'

      bar:
        echo {{x}}
      "
    })
    .used(&["x"])
    .used_parameters(&[])
    .run();
  }

  #[test]
  fn parameter_usage_self_referencing_default() {
    Test::new(indoc! {
      "
      x := 'bar'

      foo a=a:
        echo {{a}}
      "
    })
    .unresolved(&["a"])
    .used_parameters(&["a"])
    .run();
  }

  #[test]
  fn parameter_usage_tracks_body() {
    Test::new(indoc! {
      "
      foo bar:
        echo {{bar}}
      "
    })
    .used_parameters(&["bar"])
    .run();
  }

  #[test]
  fn parameters_do_not_shadow_globals_in_assignments() {
    #[track_caller]
    fn case(definition: &str) {
      let content = format!("{definition}\nbaz := bar\n");

      Test::new(&format!("bar := 'foo'\n{content}"))
        .used(&["bar"])
        .run();

      Test::new("bar := 'foo'\nimport 'foo.just'\n")
        .file("foo.just", &content)
        .used(&["bar"])
        .run();
    }

    case("foo bar:");
    case("foo(bar) := bar");
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
    .file(
      "lib.just",
      indoc! {
        "
        param := 'x'

        helper := param
        "
      },
    )
    .used(&["param"])
    .run();
  }

  #[test]
  fn recipe_with_no_parameters_or_body() {
    Test::new(indoc! {
      "
      foo:
      "
    })
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
      import 'foo.just'
      "
    })
    .file(
      "foo.just",
      indoc! {
        "
        use:
          echo {{absent}}
        "
      },
    )
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
    .run();
  }

  #[test]
  fn variable_defined_and_unused() {
    Test::new(indoc! {
      "
      foo := 'bar'
      "
    })
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
