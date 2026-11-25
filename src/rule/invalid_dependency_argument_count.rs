use super::*;

define_rule! {
  /// Checks that dependency invocations supply the correct number of arguments
  /// for the referenced recipe's signature.
  InvalidDependencyArgumentCountRule {
    id: "invalid-dependency-argument-count",
    title: "invalid dependency arguments",
    severity: Error,
  }
}

impl InvalidDependencyArgumentCountRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let lists = context.setting_enabled("lists");

    for recipe in context.recipes(Scope::Local) {
      for dependency in &recipe.dependencies {
        if let Some(recipe) = context.resolve_recipe(&dependency.name) {
          let parameters = &recipe.parameters;

          let required_parameters = parameters
            .iter()
            .filter(|parameter| recipe.is_required(parameter))
            .count();

          let has_unlimited_variadic = !lists
            && parameters.iter().any(|parameter| {
              matches!(parameter.kind, ParameterKind::Variadic(_))
            });

          let (argument_count, parameter_count) =
            (dependency.arguments.len(), parameters.len());

          if argument_count < required_parameters {
            diagnostics.push(Diagnostic::new(
              format!(
                "Dependency `{}` requires {required_parameters} {}, but {argument_count} provided",
                dependency.name,
                Count("argument", required_parameters)
              ),
              dependency.range,
            ));
          } else if !has_unlimited_variadic && argument_count > parameter_count
          {
            diagnostics.push(Diagnostic::new(
              format!(
                "Dependency `{}` accepts {parameter_count} {}, but {argument_count} provided",
                dependency.name,
                Count("argument", parameter_count)
              ),
              dependency.range,
            ));
          }
        }
      }
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::{formatdoc, indoc},
  };

  #[test]
  fn analyzer_root_recipe_overrides_import() {
    AnalyzerTest::new(indoc! {
      "
      set allow-duplicate-recipes
      foo $bar:
      baz: (foo 'bar')
      import 'foo.just'
      "
    })
    .file("foo.just", "foo $bar $baz:\n")
    .run();
  }

  #[test]
  fn recipe_dependencies_with_expressions() {
    AnalyzerTest::new(indoc! {
      "
      recipe-a param:
        echo {{param}}

      recipe-b param: (recipe-a (\"##\" + param + \"##\"))
        echo \"recipe-b called with {{param}}\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_dependencies_with_multiple_expression_arguments() {
    AnalyzerTest::new(indoc! {
      "
      recipe-a a b:
        echo {{a}} {{b}}

      recipe-b param: (recipe-a (\"1\") (\"2\"))
        echo \"recipe-b called with {{param}}\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_invocation_argument_count_correct() {
    AnalyzerTest::new(indoc! {
      "
      foo arg1 arg2=\"default\":
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo `value1`)
        echo \"bar\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_invocation_flag_accepts_no_arguments() {
    #[track_caller]
    fn case(name: &str, option: &str) {
      AnalyzerTest::new(&formatdoc! {
        "
        set unstable
        set lists
        [arg({name}, {option}, flag)]
        foo bar:
          echo {{{{bar}}}}
        baz: foo
        "
      })
      .run();
    }

    case("'bar'", "long");
    case("'bar'", "short");
    case(r#""\u{62}ar""#, "long");
    case("x'bar'", "long");
  }

  #[test]
  fn recipe_invocation_flag_argument_bounds() {
    #[track_caller]
    fn case(dependency: &str, message: Option<&'static str>) {
      let test = AnalyzerTest::new(&formatdoc! {
        "
        set unstable
        set lists
        [arg('bar', long, flag)]
        foo baz bar:
          echo {{{{baz}}}} {{{{bar}}}}
        qux: {dependency}
        "
      });

      let test = if let Some(message) = message {
        test.error(
          message,
          lsp::Range::at(5, 5, 5, 5 + u32::try_from(dependency.len()).unwrap()),
        )
      } else {
        test
      };

      test.run();
    }

    case(
      "foo",
      Some("Dependency `foo` requires 1 argument, but 0 provided"),
    );
    case("(foo 'foo')", None);
    case("(foo 'foo' 'bar')", None);
    case(
      "(foo 'foo' 'bar' 'baz')",
      Some("Dependency `foo` accepts 2 arguments, but 3 provided"),
    );
  }

  #[test]
  fn recipe_invocation_missing_args() {
    AnalyzerTest::new(indoc! {
      "
      foo arg1 arg2:
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo)
        echo \"bar\"
      "
    })
    .error(
      "Dependency `foo` requires 2 arguments, but 0 provided",
      lsp::Range::at(3, 5, 3, 10),
    )
    .run();
  }

  #[test]
  fn recipe_invocation_one_or_more_variadic_requires_argument() {
    AnalyzerTest::new(indoc! {
      "
      foo +args:
        echo \"{{args}}\"

      bar: (foo)
        echo \"bar\"
      "
    })
    .error(
      "Dependency `foo` requires 1 argument, but 0 provided",
      lsp::Range::at(3, 5, 3, 10),
    )
    .run();
  }

  #[test]
  fn recipe_invocation_too_few_args() {
    AnalyzerTest::new(indoc! {
      "
      foo arg1 arg2:
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo `value1`)
        echo \"bar\"
      "
    })
    .error(
      "Dependency `foo` requires 2 arguments, but 1 provided",
      lsp::Range::at(3, 5, 3, 19),
    )
    .run();
  }

  #[test]
  fn recipe_invocation_too_many_args() {
    AnalyzerTest::new(indoc! {
      "
      foo arg1:
        echo \"{{arg1}}\"

      bar: (foo `value1` `value2` `value3`)
        echo \"bar\"
      "
    })
    .error(
      "Dependency `foo` accepts 1 argument, but 3 provided",
      lsp::Range::at(3, 5, 3, 37),
    )
    .run();
  }

  #[test]
  fn recipe_invocation_valid_variable() {
    AnalyzerTest::new(indoc! {
      "
      wow := `foo`

      foo arg1:
        echo \"{{arg1}}\"

      bar: (foo wow)
        echo \"bar\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_invocation_variadic_params() {
    AnalyzerTest::new(indoc! {
      "
      foo arg1 +args:
        echo \"{{arg1}} {{args}}\"

      bar: (foo 'value1' 'value2' 'value3')
        echo \"bar\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_invocation_variadic_params_reject_extra_arguments_with_lists() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      foo arg1 +args:
        echo \"{{arg1}} {{args}}\"

      bar: (foo 'value1' 'value2' 'value3')
        echo \"bar\"
      "
    })
    .error(
      "Dependency `foo` accepts 2 arguments, but 3 provided",
      lsp::Range::at(6, 5, 6, 37),
    )
    .run();
  }

  #[test]
  fn recipe_invocation_zero_or_more_variadic_accepts_no_arguments() {
    AnalyzerTest::new(indoc! {
      "
      foo *args:
        echo \"{{args}}\"

      bar: (foo)
        echo \"bar\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_invocation_zero_or_more_variadic_with_min_requires_argument() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      [arg('bar', min='2')]
      foo *bar:
        echo {{bar}}

      baz: foo
      "
    })
    .error(
      "Dependency `foo` requires 1 argument, but 0 provided",
      lsp::Range::at(7, 5, 7, 8),
    )
    .run();
  }
}
