use super::*;

define_rule! {
  /// Finds non-exported global variables that are never referenced in the
  /// document or any of its imports.
  UnusedVariableRule {
    id: "unused-variable",
    message: "unused variable",
    run(context) {
      let mut diagnostics = Vec::new();

      let exported = context.setting_enabled("export");

      for (variable_name, is_used) in &context.references().variable_usage {
        if variable_name.starts_with('_') || *is_used {
          continue;
        }

        let Some(variable) = context
          .local_declarations(context.variables())
          .filter(|variable| variable.name.value == *variable_name)
          .last()
        else {
          continue;
        };

        if variable.eager
          || variable.export
          || exported
          || variable
            .attributes
            .iter()
            .any(|attribute| attribute.name.value == "private")
        {
          continue;
        }

        diagnostics.push(Diagnostic::warning(
          format!("Variable `{variable_name}` appears unused"),
          variable.name.range,
        ));
      }

      diagnostics
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_uses_imported_variable_usage() {
    AnalyzerTest::new(indoc! {
      "
      import? 'foo.just'

      lib_var := 'x'

      foo: use
      "
    })
    .file(
      "foo.just",
      indoc! {
        "
        use:
          echo {{lib_var}}
        "
      },
    )
    .run();
  }

  #[test]
  fn attribute_variable_argument_usage() {
    AnalyzerTest::new(indoc! {
      "
      foo := 'bar'

      [env('bar', foo)]
      bar:
        echo bar
      "
    })
    .run();
  }

  #[test]
  fn cross_parameter_default_references_preceding_parameter() {
    AnalyzerTest::new(indoc! {
      "
      a := 'foo'

      foo a b=a:
        echo {{ a }} {{ b }}
      "
    })
    .warning("Variable `a` appears unused", lsp::Range::at(0, 0, 0, 1))
    .run();
  }

  #[test]
  fn exported_variables_not_warned() {
    AnalyzerTest::new(indoc! {
      "
      foo := \"unused value\"
      export bar := \"exported but unused\"
      baz := \"used value\"

      recipe:
        echo {{ baz }}
      "
    })
    .warning("Variable `foo` appears unused", lsp::Range::at(0, 0, 0, 3))
    .run();
  }

  #[test]
  fn format_strings_mark_variables_as_used() {
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
  fn private_and_eager_variables_are_not_unused() {
    AnalyzerTest::new(indoc! {
      "
      [private]
      foo := 'bar'
      eager baz := 'qux'
      "
    })
    .run();
  }

  #[test]
  fn set_export_suppresses_unused_variable_warnings() {
    AnalyzerTest::new(indoc! {
      "
      set export

      foo := 'bar'
      baz := 'qux'

      recipe:
        echo $foo
      "
    })
    .run();
  }

  #[test]
  fn underscore_suppresses_unused_variable_warnings() {
    AnalyzerTest::new(indoc! {
      "
      _ := 'foo'
      _bar := 'baz'

      recipe:
        echo 'Hello!'
      "
    })
    .run();
  }

  #[test]
  fn unexported_names_are_not_variables() {
    AnalyzerTest::new(indoc! {
      "
      foo := \"unused value\"
      unexport BAR
      baz := \"used value\"

      recipe:
        echo {{ baz }}
      "
    })
    .warning("Variable `foo` appears unused", lsp::Range::at(0, 0, 0, 3))
    .run();
  }

  #[test]
  fn used_variables_no_warnings() {
    AnalyzerTest::new(indoc! {
      "
      foo := \"used in recipe\"
      bar := \"used as dependency arg\"

      another arg:
        echo {{ arg }}

      recipe: (another bar)
        echo {{ foo }}
      "
    })
    .run();
  }

  #[test]
  fn variables_and_parameters_same_name() {
    AnalyzerTest::new(indoc! {
      "
      param := \"variable value\"
      other := \"other value\"

      recipe param:
        # This should reference the parameter, not the variable
        echo {{ param }}
        echo {{ other }}
      "
    })
    .warning(
      "Variable `param` appears unused",
      lsp::Range::at(0, 0, 0, 5),
    )
    .run();
  }

  #[test]
  fn variables_used_after_hash_in_command() {
    AnalyzerTest::new(indoc! {
      "
      flake := \"testflake\"
      output := \"testoutput\"

      test:
        darwin-rebuild switch --flake {{ flake }}#{{ output }}
      "
    })
    .run();
  }

  #[test]
  fn variables_used_in_dependency_args() {
    AnalyzerTest::new(indoc! {
      "
      used_arg := \"value\"
      unused_var := \"not used\"

      recipe: (another used_arg)
        echo \"something\"

      another arg:
        echo {{ arg }}
      "
    })
    .warning(
      "Variable `unused_var` appears unused",
      lsp::Range::at(1, 0, 1, 10),
    )
    .run();
  }

  #[test]
  fn variables_used_in_multiple_recipes() {
    AnalyzerTest::new(indoc! {
      "
      shared := \"shared value\"
      only_in_first := \"first value\"
      only_in_second := \"second value\"
      never_used := \"unused\"

      first:
        echo {{ shared }}
        echo {{ only_in_first }}

      second:
        echo {{ shared }}
        echo {{ only_in_second }}
      "
    })
    .warning(
      "Variable `never_used` appears unused",
      lsp::Range::at(3, 0, 3, 10),
    )
    .run();
  }

  #[test]
  fn variables_used_in_recipe_default_parameters() {
    AnalyzerTest::new(indoc! {
      "
      param_value := \"value\"

      recipe arg=param_value:
        echo {{ arg }}
      "
    })
    .run();
  }

  #[test]
  fn variables_used_in_recipe_dependencies() {
    AnalyzerTest::new(indoc! {
      "
      param_value := \"value\"
      unused := \"unused\"

      recipe arg=\"default\": (another param_value)
        echo {{ arg }}

      another arg:
        echo {{ arg }}
      "
    })
    .warning(
      "Variable `unused` appears unused",
      lsp::Range::at(1, 0, 1, 6),
    )
    .run();
  }

  #[test]
  fn variables_used_in_starred_dependency_args() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      used := \"value\"
      unused := \"not used\"

      recipe: *(another *used)
        echo \"foo\"

      another arg:
        echo {{ arg }}
      "
    })
    .warning(
      "Variable `unused` appears unused",
      lsp::Range::at(4, 0, 4, 6),
    )
    .run();
  }

  #[test]
  fn warn_for_unused_variables() {
    AnalyzerTest::new(indoc! {
      "
      foo := \"unused value\"
      bar := \"used value\"

      recipe:
        echo {{ bar }}
      "
    })
    .warning("Variable `foo` appears unused", lsp::Range::at(0, 0, 0, 3))
    .run();
  }
}
