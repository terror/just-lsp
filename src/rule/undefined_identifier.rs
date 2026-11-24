use super::*;

define_rule! {
  /// Reports expressions that reference variables or parameters which are not
  /// defined and aren't builtins.
  UndefinedIdentifierRule {
    id: "undefined-identifier",
    message: "undefined identifier",
    run(context) {
      let mut diagnostics = Vec::new();

      for (identifier, suggestion) in &context.references().unresolved_identifiers {
        let message = match suggestion {
          Some(suggestion) => format!(
            "Variable `{}` not found. Did you mean `{suggestion}`?",
            identifier.value,
          ),
          None => format!("Variable `{}` not found", identifier.value),
        };

        let quickfix = suggestion.as_deref().map(|suggestion| {
          Quickfix::replacement(identifier, suggestion)
        });

        diagnostics.push(
          Diagnostic::error(message, identifier.range).quickfix(quickfix),
        );
      }

      diagnostics
    }
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::{formatdoc, indoc},
  };

  #[test]
  fn attribute_expression_scope() {
    #[track_caller]
    fn case(attribute: &str, error: Option<(&'static str, lsp::Range)>) {
      let test = AnalyzerTest::new(&formatdoc! {
        "
        {attribute}
        foo bar baz:
          echo {{{{bar}}}} {{{{baz}}}}
        "
      });

      match error {
        Some((message, range)) => test.error(message, range),
        None => test,
      }
      .run();
    }

    case(
      "[doc(bar)]",
      Some(("Variable `bar` not found", lsp::Range::at(0, 5, 0, 8))),
    );

    case("[arg('baz', long, value=bar)]", None);

    case(
      "[arg('bar', long, value=baz)]",
      Some(("Variable `baz` not found", lsp::Range::at(0, 24, 0, 27))),
    );

    case("[working-directory(bar)]", None);
  }

  #[test]
  fn default_parameter_expression_functions() {
    AnalyzerTest::new(indoc! {
      "
      build version=uppercase(\"1.0.0\"):
        echo {{ version }}
      "
    })
    .run();
  }

  #[test]
  fn default_parameter_expression_with_env_call() {
    AnalyzerTest::new(indoc! {
      "
      build target=(env('TARGET', 'debug')):
        echo {{ target }}
      "
    })
    .run();
  }

  #[test]
  fn format_strings_with_function_calls() {
    AnalyzerTest::new(indoc! {
      r#"
      info := f'arch: {{arch()}}'
      more := f'upper: {{uppercase("foo",)}}'
      foo:
        echo {{info}}
        echo {{more}}
      "#
    })
    .run();
  }

  #[test]
  fn format_strings_with_undefined_variables() {
    AnalyzerTest::new(indoc! {
      r"
      greeting := f'Hello, {{undefined_var}}!'
      foo:
        echo {{greeting}}
      "
    })
    .error(
      "Variable `undefined_var` not found",
      lsp::Range::at(0, 23, 0, 36),
    )
    .run();
  }

  #[test]
  fn format_strings_with_valid_variables() {
    AnalyzerTest::new(indoc! {
      r#"
      name := "world"
      greeting := f'Hello, {{name}}!'
      foo:
        echo {{greeting}}
      "#
    })
    .run();
  }

  #[test]
  fn parameter_default_references_preceding_parameter() {
    AnalyzerTest::new(indoc! {
      "
      @binstall crate bin=crate:
        which {{bin}} 2>&1 >/dev/null || cargo binstall -y {{crate}}
      "
    })
    .run();
  }

  #[test]
  fn parameters_do_not_leak_into_assignments() {
    #[track_caller]
    fn case(definition: &str) {
      AnalyzerTest::new(&format!(
        "set unstable\n{definition}bar := foobar + foobaz\n"
      ))
      .warning("Variable `bar` appears unused", lsp::Range::at(3, 0, 3, 3))
      .error("Variable `foobar` not found", lsp::Range::at(3, 7, 3, 13))
      .error("Variable `foobaz` not found", lsp::Range::at(3, 16, 3, 22))
      .run();
    }

    case("_foo foobar:\n  echo {{foobar}}\n");
    case("_foo(foobar) := foobar\n\n");
  }

  #[test]
  fn parenthesized_expression_default_uses_global_variable() {
    AnalyzerTest::new(indoc! {
      "
      foo := 'foo'
      bar := 'bar'

      recipe x=(foo + bar):
        echo {{ x }}
      "
    })
    .run();
  }

  #[test]
  fn recipe_invocation_unknown_variable() {
    AnalyzerTest::new(indoc! {
      "
      foo arg1:
        echo {{ arg1 }}

      bar: (foo wow)
        echo \"bar\"
      "
    })
    .error("Variable `wow` not found", lsp::Range::at(3, 10, 3, 13))
    .run();
  }

  #[test]
  fn settings_ignore_comments() {
    AnalyzerTest::new(indoc! {
      "
      set ignore-comments

      foo:
        # {{bar}}
        echo \\
        # {{baz}}
        @# {{qux}}

      [script]
      bar:
        # {{bob}}
      "
    })
    .error("Variable `baz` not found", lsp::Range::at(5, 6, 5, 9))
    .error("Variable `qux` not found", lsp::Range::at(6, 7, 6, 10))
    .error("Variable `bob` not found", lsp::Range::at(10, 6, 10, 9))
    .run();
  }

  #[test]
  fn shadowed_parameter_default_uses_global_variable() {
    AnalyzerTest::new(indoc! {
      "
      a := 'default a'

      b a=a:
        echo {{ a }}
      "
    })
    .run();
  }

  #[test]
  fn should_recognize_recipe_parameters_in_dependency_arguments() {
    AnalyzerTest::new(indoc! {
      "
      other-recipe var=\"else\":
        echo {{ var }}

      test var=\"something\": (other-recipe var)
      "
    })
    .run();
  }

  #[test]
  fn unknown_default_recipe_parameter_reference() {
    AnalyzerTest::new(indoc! {
      "
      recipe arg=foo:
        echo {{ arg }}
      "
    })
    .error("Variable `foo` not found", lsp::Range::at(0, 11, 0, 14))
    .run();
  }

  #[test]
  fn unreferenced_variable_in_expression() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo {{ var }}
      "
    })
    .error("Variable `var` not found", lsp::Range::at(1, 10, 1, 13))
    .run();
  }

  #[test]
  fn user_defined_function_body_references_variable() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      base := \"hello\"

      foo(x) := base + x

      bar:
        echo {{foo('bar')}}
      "
    })
    .run();
  }

  #[test]
  fn user_defined_function_body_unknown_identifier() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      foo(x) := x + unknown

      bar:
        echo {{foo('bar')}}
      "
    })
    .error("Variable `unknown` not found", lsp::Range::at(2, 14, 2, 21))
    .run();
  }

  #[test]
  fn user_defined_function_name_is_not_variable() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      foo() := 'bar'
      baz := foo

      qux:
        echo {{foo()}} {{baz}}
      "
    })
    .error("Variable `foo` not found", lsp::Range::at(3, 7, 3, 10))
    .run();
  }

  #[test]
  fn user_defined_function_parameters_not_unresolved() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      foo(x) := x + \"!\"

      bar:
        echo {{foo('bar')}}
      "
    })
    .run();
  }
}
