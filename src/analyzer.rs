use super::*;

static RULES: &[&dyn Rule] = &[
  &SyntaxRule,
  &MissingRecipeForAliasRule,
  &DuplicateAliasRule,
  &AliasRecipeConflictRule,
  &UnknownAttributeRule,
  &AttributeArgumentsRule,
  &AttributeInvalidTargetRule,
  &AttributeTargetSupportRule,
  &ScriptShebangConflictRule,
  &DuplicateDefaultAttributeRule,
  &UnknownFunctionRule,
  &FunctionArgumentsRule,
  &RecipeParameterRule,
  &MixedIndentationRule,
  &InconsistentIndentationRule,
  &DuplicateRecipeRule,
  &RecipeDependencyCycleRule,
  &MissingDependencyRule,
  &DependencyArgumentRule,
  &UnknownSettingRule,
  &InvalidSettingKindRule,
  &DuplicateSettingRule,
  &DuplicateVariableRule,
  &UndefinedIdentifierRule,
  &UnusedVariableRule,
  &UnusedParameterRule,
];

#[derive(Debug)]
pub(crate) struct Analyzer<'a> {
  document: &'a Document,
}

impl<'a> Analyzer<'a> {
  /// Analyzes the document and returns a list of diagnostics.
  pub(crate) fn analyze(&self) -> Vec<lsp::Diagnostic> {
    let context = RuleContext::new(self.document);
    RULES.iter().flat_map(|rule| rule.run(&context)).collect()
  }

  /// Creates a new analyzer for the given document.
  #[must_use]
  pub(crate) fn new(document: &'a Document) -> Self {
    Self { document }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  #[derive(Debug)]
  struct Test {
    document: Document,
    messages: Vec<(String, Option<lsp::DiagnosticSeverity>)>,
  }

  impl Test {
    fn error(self, message: &str) -> Self {
      Self {
        messages: self
          .messages
          .into_iter()
          .chain([(message.to_owned(), Some(lsp::DiagnosticSeverity::ERROR))])
          .collect(),
        ..self
      }
    }

    fn new(content: &str) -> Self {
      Self {
        document: Document::try_from(lsp::DidOpenTextDocumentParams {
          text_document: lsp::TextDocumentItem {
            uri: lsp::Url::parse("file:///test.just").unwrap(),
            language_id: "just".to_string(),
            version: 1,
            text: content.to_string(),
          },
        })
        .unwrap(),
        messages: Vec::new(),
      }
    }

    fn run(self) {
      let analyzer = Analyzer::new(&self.document);

      let messages = analyzer
        .analyze()
        .into_iter()
        .map(|d| (d.message, d.severity))
        .collect::<Vec<(String, Option<lsp::DiagnosticSeverity>)>>();

      assert_eq!(messages, self.messages);
    }

    fn warning(self, message: &str) -> Self {
      Self {
        messages: self
          .messages
          .into_iter()
          .chain([(message.to_owned(), Some(lsp::DiagnosticSeverity::WARNING))])
          .collect(),
        ..self
      }
    }
  }

  #[test]
  fn aliases_basic() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      alias bar := foo
      "
    })
    .run();
  }

  #[test]
  fn aliases_duplicate() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      alias bar := foo
      alias bar := foo
      alias bar := foo
      "
    })
    .error("Duplicate alias `bar`")
    .error("Duplicate alias `bar`")
    .run();
  }

  #[test]
  fn aliases_missing_recipe() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      alias bar := baz
      "
    })
    .error("Recipe `baz` not found")
    .run();
  }

  #[test]
  fn alias_recipe_conflict_recipe_then_alias() {
    Test::new(indoc! {
      "
      other:
        echo \"other\"

      t:
        echo \"recipe\"

      alias t := other
      "
    })
    .error("Recipe `t` is redefined as an alias")
    .run();
  }

  #[test]
  fn alias_recipe_conflict_alias_then_recipe() {
    Test::new(indoc! {
      "
      alias t := other

      other:
        echo \"other\"

      t:
        echo \"recipe\"
      "
    })
    .error("Alias `t` is redefined as a recipe")
    .run();
  }

  #[test]
  fn analyze_complete() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: missing
        echo \"bar\"

      alias baz := nonexistent
      "
    })
    .error("Recipe `nonexistent` not found")
    .error("Recipe `missing` not found")
    .run();
  }

  #[test]
  fn attributes_correct() {
    Test::new(indoc! {
      "
      [no-cd]
      [linux]
      [macos]
      foo:
        echo \"foo\"

      [doc('Recipe documentation')]
      bar:
        echo \"bar\"

      [default]
      baz:
        echo \"baz\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_duplicate_default_between_recipes() {
    Test::new(indoc! {
      "
      [default]
      check:
        echo \"check\"

      [default]
      ci:
        echo \"ci\"
      "
    })
    .error(
      "Recipe `ci` has duplicate `[default]` attribute, which may only appear once per module",
    )
    .run();
  }

  #[test]
  fn attributes_duplicate_default_on_same_recipe() {
    Test::new(indoc! {
      "
      [default]
      [default]
      build:
        echo \"build\"
      "
    })
    .error(
      "Recipe `build` has duplicate `[default]` attribute, which may only appear once per module",
    )
    .run();
  }

  #[test]
  fn attributes_extra_arguments() {
    Test::new(indoc! {
      "
      [linux('invalid')]
      foo:
        echo \"foo\"
      "
    })
    .error("Attribute `linux` got 1 argument but takes 0 arguments")
    .run();

    Test::new(indoc! {
      "
      [default('invalid')]
      foo:
        echo \"foo\"
      "
    })
    .error("Attribute `default` got 1 argument but takes 0 arguments")
    .run();
  }

  #[test]
  fn attributes_missing_arguments() {
    Test::new(indoc! {
      "
      [doc]
      foo:
        echo \"foo\"
      "
    })
    .error("Attribute `doc` got 0 arguments but takes 1 argument")
    .run();
  }

  #[test]
  fn escaped_braces_are_treated_as_literal_text() {
    Test::new(indoc! {
      "
      test:
        echo \"{{{{hello}}\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_no_parameters_needed() {
    Test::new(indoc! {
      "
      [script]
      foo:
        echo \"foo\"
      "
    })
    .run();

    Test::new(indoc! {
      "
      [confirm]
      foo:
        echo \"foo\"
      "
    })
    .run();

    Test::new(indoc! {
      "
      [default]
      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_unknown() {
    Test::new(indoc! {
      "
      [unknown_attribute]
      foo:
        echo \"foo\"
      "
    })
    .error("Unknown attribute `unknown_attribute`")
    .run();
  }

  #[test]
  fn script_attribute_with_shebang_conflict() {
    Test::new(indoc! {
      "
      [script]
      publish:
        #!/usr/bin/env bash
        echo \"publish\"
      "
    })
    .error("Recipe `publish` has both shebang line and `[script]` attribute")
    .run();
  }

  #[test]
  fn script_attribute_without_shebang_is_allowed() {
    Test::new(indoc! {
      "
      [script]
      publish:
        echo \"publish\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_wrong_target() {
    Test::new(indoc! {
      "
      [group: 'foo']
      alias f := foo

      foo:
        echo \"foo\"
      "
    })
    .error("Attribute `group` cannot be applied to alias target")
    .run();
  }

  #[test]
  fn attributes_invalid_inline() {
    Test::new(indoc! {
      "
      [group: 'foo', foo]
      foo:
        echo \"foo\"
      "
    })
    .error("Unknown attribute `foo`")
    .run();
  }

  #[test]
  fn attributes_inline_parameters_focused() {
    Test::new(indoc! {
      "
      [group: 'foo', no-cd]
      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn module_attributes_group() {
    Test::new(indoc! {
      "
      [group: 'tools']
      mod foo
      "
    })
    .run();
  }

  #[test]
  fn attributes_more_arguments_than_required() {
    Test::new(indoc! {
      "
      [group('foo', 'bar')]
      foo:
        echo \"foo\"
      "
    })
    .error("Attribute `group` got 2 arguments but takes 1 argument")
    .run();
  }

  #[test]
  fn attributes_on_assignments() {
    Test::new(indoc! {
      "
      [private]
      secret := \"secret value\"

      [private]
      _db_url := \"postgres://user:pass@host:port/db\"

      public_var := \"public value\"

      test:
        echo {{ secret }}
        echo {{ _db_url }}
        echo {{ public_var }}
      "
    })
    .run();
  }

  #[test]
  fn attributes_on_exported_assignments() {
    Test::new(indoc! {
      "
      [private]
      export PATH := '/usr/local/bin'
      "
    })
    .run();
  }

  #[test]
  fn function_calls_correct() {
    Test::new(indoc! {
      "
      foo:
        echo {{ arch() }}
        echo {{ join(\"a\", \"b\", \"c\") }}
      "
    })
    .run();
  }

  #[test]
  fn function_calls_too_few_args() {
    Test::new(indoc! {
      "
      foo:
        echo {{ replace() }}
      "
    })
    .error("Function `replace` requires at least 3 arguments, but 0 provided")
    .run();
  }

  #[test]
  fn function_calls_too_many_args() {
    Test::new(indoc! {
      "
      foo:
        echo {{ uppercase(\"hello\", \"extra\") }}
      "
    })
    .error("Function `uppercase` accepts 1 argument, but 2 provided")
    .run();
  }

  #[test]
  fn function_calls_unknown() {
    Test::new(indoc! {
      "
      foo:
        echo {{ unknown_function() }}
      "
    })
    .error("Unknown function `unknown_function`")
    .run();
  }

  #[test]
  fn function_calls_nested() {
    Test::new(indoc! {
      "
      foo:
        echo {{ replace(parent_directory('~/.config/nvim/init.lua'), '.', 'dot-') }}
      "
    })
    .run();
  }

  #[test]
  fn parser_errors_invalid() {
    Test::new(indoc! {
      "
      foo
        echo \"foo\"
      "
    })
    .error("Syntax error")
    .run();
  }

  #[test]
  fn recipe_mixed_indentation_between_lines() {
    Test::new(indoc! {
      "
      foo:
      \techo \"foo\"
        echo \"bar\"
      "
    })
    .error("Recipe `foo` mixes tabs and spaces for indentation")
    .run();
  }

  #[test]
  fn recipe_mixed_indentation_single_line_mix() {
    Test::new(indoc! {
      "
      foo:
   \t  echo \"foo\"
      "
    })
    .error("Recipe `foo` mixes tabs and spaces for indentation")
    .run();
  }

  #[test]
  fn recipe_inconsistent_indentation_between_lines() {
    Test::new("foo:\n        echo \"foo\"\n  echo \"bar\"\n")
    .error(
      "Recipe line has inconsistent leading whitespace. Recipe started with `␠␠␠␠␠␠␠␠` but found line with `␠␠`",
    )
    .run();
  }

  #[test]
  fn recipe_consistent_indentation() {
    Test::new("foo:\n  echo \"foo\"\n  echo \"bar\"\n").run();
  }

  #[test]
  fn parser_errors_valid() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_dependencies_correct() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_dependencies_missing() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: baz
        echo \"bar\"
      "
    })
    .error("Recipe `baz` not found")
    .run();
  }

  #[test]
  fn recipe_dependencies_multiple_missing() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: missing1 missing2
        echo \"bar\"
      "
    })
    .error("Recipe `missing1` not found")
    .error("Recipe `missing2` not found")
    .run();
  }

  #[test]
  fn recipe_invocation_argument_count_correct() {
    Test::new(indoc! {
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
  fn recipe_invocation_missing_args() {
    Test::new(indoc! {
      "
      foo arg1 arg2:
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo)
        echo \"bar\"
      "
    })
    .error("Dependency `foo` requires 2 arguments, but 0 provided")
    .run();
  }

  #[test]
  fn recipe_invocation_too_few_args() {
    Test::new(indoc! {
      "
      foo arg1 arg2:
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo `value1`)
        echo \"bar\"
      "
    })
    .error("Dependency `foo` requires 2 arguments, but 1 provided")
    .run();
  }

  #[test]
  fn recipe_invocation_too_many_args() {
    Test::new(indoc! {
      "
      foo arg1:
        echo \"{{arg1}}\"

      bar: (foo `value1` `value2` `value3`)
        echo \"bar\"
      "
    })
    .error("Dependency `foo` accepts 1 argument, but 3 provided")
    .run();
  }

  #[test]
  fn recipe_invocation_unknown_variable() {
    Test::new(indoc! {
      "
      foo arg1:
        echo {{ arg1 }}

      bar: (foo wow)
        echo \"bar\"
      "
    })
    .error("Variable `wow` not found")
    .run();
  }

  #[test]
  fn recipe_invocation_valid_variable() {
    Test::new(indoc! {
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
    Test::new(indoc! {
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
  fn recipe_dependencies_with_expressions() {
    Test::new(indoc! {
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
    Test::new(indoc! {
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
  fn recipe_parameters_defaults_all() {
    Test::new(indoc! {
      "
      recipe_with_defaults arg1=\"first\" arg2=\"second\":
        echo \"{{arg1}} {{arg2}}\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_parameters_duplicate() {
    Test::new(indoc! {
      "
      recipe_with_duplicate_param arg1 arg1:
        echo \"{{arg1}}\"
      "
    })
    .error("Duplicate parameter `arg1`")
    .run();
  }

  #[test]
  fn recipe_parameters_order() {
    Test::new(indoc! {
      "
      recipe_with_param_order arg1=\"default\" arg2:
        echo \"{{arg1}} {{arg2}}\"
      "
    })
    .error("Required parameter `arg2` follows a parameter with a default value")
    .run();
  }

  #[test]
  fn recipe_parameters_valid() {
    Test::new(indoc! {
      "
      valid_recipe arg1 arg2=\"default\":
        echo \"{{arg1}} {{arg2}}\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_parameters_variadic() {
    Test::new(indoc! {
      "
      recipe_with_variadic arg1=\"default\" +args:
        echo \"{{arg1}} {{args}}\"
      "
    })
    .run();
  }

  #[test]
  fn settings_boolean_shorthand() {
    Test::new(indoc! {
      "
      set export

      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn settings_boolean_type_correct() {
    Test::new(indoc! {
      "
      set export := true
      set dotenv-load := false

      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn settings_boolean_type_error() {
    Test::new(indoc! {
      "
      set export := 'foo'

      foo:
        echo \"foo\"
      "
    })
    .error("Setting `export` expects a boolean value")
    .run();
  }

  #[test]
  fn settings_duplicate() {
    Test::new(indoc! {
      "
      set export := true
      set shell := [\"bash\", \"-c\"]
      set export := false

      foo:
        echo \"foo\"
      "
    })
    .error("Duplicate setting `export`")
    .run();
  }

  #[test]
  fn settings_multiple_errors() {
    Test::new(indoc! {
      "
      set unknown-setting := true
      set export := false
      set shell := ['bash']
      set export := false

      foo:
        echo \"foo\"
      "
    })
    .error("Unknown setting `unknown-setting`")
    .error("Duplicate setting `export`")
    .run();
  }

  #[test]
  fn settings_string_type_correct() {
    Test::new(indoc! {
      "
      set dotenv-path := \".env.development\"

      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn settings_string_type_error() {
    Test::new(indoc! {
      "
      set dotenv-path := true

      foo:
        echo \"foo\"
      "
    })
    .error("Setting `dotenv-path` expects a string value")
    .run();
  }

  #[test]
  fn settings_unknown() {
    Test::new(indoc! {
      "
      set unknown-setting := true

      foo:
        echo \"foo\"
      "
    })
    .error("Unknown setting `unknown-setting`")
    .run();
  }

  #[test]
  fn should_recognize_recipe_parameters_in_dependency_arguments() {
    Test::new(indoc! {
      "
      other-recipe var=\"else\":
        echo {{ var }}

      test var=\"something\": (other-recipe var)
      "
    })
    .run();
  }

  #[test]
  fn unreferenced_variable_in_expression() {
    Test::new(indoc! {
      "
      foo:
        echo {{ var }}
      "
    })
    .error("Variable `var` not found")
    .run();
  }

  #[test]
  fn warn_for_unused_non_exported_recipe_parameters() {
    Test::new(indoc! {
      "
      foo bar:
        echo foo
      "
    })
    .warning("Parameter `bar` appears unused")
    .run();

    Test::new(indoc! {
      "
      foo $bar:
        echo foo
      "
    })
    .run();

    Test::new(indoc! {
      "
      set export := false

      foo bar:
        echo foo
      "
    })
    .warning("Parameter `bar` appears unused")
    .run();

    Test::new(indoc! {
      "
      set export

      foo bar:
        echo foo
      "
    })
    .run();
  }

  #[test]
  fn duplicate_recipe_names() {
    Test::new(indoc! {
      "
      foo:
        echo foo

      foo:
        echo foo

      foo:
        echo foo
      "
    })
    .error("Duplicate recipe name `foo`")
    .error("Duplicate recipe name `foo`")
    .run();
  }

  #[test]
  fn warn_for_unused_variables() {
    Test::new(indoc! {
      "
      foo := \"unused value\"
      bar := \"used value\"

      recipe:
        echo {{ bar }}
      "
    })
    .warning("Variable `foo` appears unused")
    .run();
  }

  #[test]
  fn duplicate_variable_assignments() {
    Test::new(indoc! {
      "
      foo := \"one\"
      foo := \"two\"

      recipe:
        echo {{ foo }}
      "
    })
    .error("Duplicate variable `foo`")
    .run();
  }

  #[test]
  fn duplicate_variable_assignments_allowed_via_setting() {
    Test::new(indoc! {
      "
      set allow-duplicate-variables := true

      foo := \"one\"
      foo := \"two\"

      recipe:
        echo {{ foo }}
      "
    })
    .run();
  }

  #[test]
  fn duplicate_recipe_names_allowed_via_setting() {
    Test::new(indoc! {
      "
      set allow-duplicate-recipes := true

      foo:
        echo foo

      [linux]
      foo:
        echo foo on linux
      "
    })
    .run();
  }

  #[test]
  fn used_variables_no_warnings() {
    Test::new(indoc! {
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
  fn variables_used_in_recipe_dependencies() {
    Test::new(indoc! {
      "
      param_value := \"value\"
      unused := \"unused\"

      recipe arg=\"default\": (another param_value)
        echo {{ arg }}

      another arg:
        echo {{ arg }}
      "
    })
    .warning("Variable `unused` appears unused")
    .run();
  }

  #[test]
  fn variables_used_after_hash_in_command() {
    Test::new(indoc! {
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
  fn variables_used_in_recipe_default_parameters() {
    Test::new(indoc! {
      "
      param_value := \"value\"

      recipe arg=param_value:
        echo {{ arg }}
      "
    })
    .run();
  }

  #[test]
  fn default_parameter_expression_functions() {
    Test::new(indoc! {
      "
      build version=uppercase(\"1.0.0\"):
        echo {{ version }}
      "
    })
    .run();
  }

  #[test]
  fn default_parameter_expression_with_env_call() {
    Test::new(indoc! {
      "
      build target=(env('TARGET', 'debug')):
        echo {{ target }}
      "
    })
    .run();
  }

  #[test]
  fn unknown_default_recipe_parameter_reference() {
    Test::new(indoc! {
      "
      recipe arg=foo:
        echo {{ arg }}
      "
    })
    .error("Variable `foo` not found")
    .run();
  }

  #[test]
  fn variables_used_in_dependency_args() {
    Test::new(indoc! {
      "
      used_arg := \"value\"
      unused_var := \"not used\"

      recipe: (another used_arg)
        echo \"something\"

      another arg:
        echo {{ arg }}
      "
    })
    .warning("Variable `unused_var` appears unused")
    .run();
  }

  #[test]
  fn variables_and_parameters_same_name() {
    Test::new(indoc! {
      "
      param := \"variable value\"
      other := \"other value\"

      recipe param:
        # This should reference the parameter, not the variable
        echo {{ param }}
        echo {{ other }}
      "
    })
    .warning("Variable `param` appears unused")
    .run();
  }

  #[test]
  fn variables_used_in_multiple_recipes() {
    Test::new(indoc! {
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
    .warning("Variable `never_used` appears unused")
    .run();
  }

  #[test]
  fn exported_variables_not_warned() {
    Test::new(indoc! {
      "
      foo := \"unused value\"
      export bar := \"exported but unused\"
      baz := \"used value\"

      recipe:
        echo {{ baz }}
      "
    })
    .warning("Variable `foo` appears unused")
    .run();
  }

  #[test]
  fn os_specific_duplicate_recipes() {
    Test::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      [windows]
      build:
        echo \"Building on Windows\"

      [macos]
      build:
        echo \"Building on macOS\"
      "
    })
    .run();
  }

  #[test]
  fn duplicate_recipes_with_same_os_attribute() {
    Test::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux version 1\"

      [linux]
      build:
        echo \"Building on Linux version 2\"
      "
    })
    .error("Duplicate recipe name `build`")
    .run();
  }

  #[test]
  fn mixed_os_specific_and_regular_recipe() {
    Test::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      build:
        echo \"Building on any OS\"
      "
    })
    .error("Duplicate recipe name `build`")
    .run();
  }

  #[test]
  fn windows_recipe_conflicts_with_default() {
    Test::new(indoc! {
      "
      [windows]
      build:
        echo \"Building on Windows\"

      build:
        echo \"Building on every OS\"
      "
    })
    .error("Duplicate recipe name `build`")
    .run();
  }

  #[test]
  fn unix_macos_conflicts() {
    Test::new(indoc! {
      "
      [unix]
      build:
        echo \"Building on Unix systems\"

      [macos]
      build:
        echo \"Building on macOS specifically\"
      "
    })
    .error("Duplicate recipe name `build`")
    .run();
  }

  #[test]
  fn linux_openbsd_no_conflict() {
    Test::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      [openbsd]
      build:
        echo \"Building on OpenBSD\"
      "
    })
    .run();
  }

  #[test]
  fn linux_unix_conflict() {
    Test::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      [unix]
      build:
        echo \"Building on Unix systems\"
      "
    })
    .error("Duplicate recipe name `build`")
    .run();
  }

  #[test]
  fn openbsd_macos_no_conflict() {
    Test::new(indoc! {
      "
      [openbsd]
      build:
        echo \"Building on OpenBSD\"

      [macos]
      build:
        echo \"Building on macOS\"
      "
    })
    .run();
  }

  #[test]
  fn all_four_os_groups_no_conflict() {
    Test::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      [macos]
      build:
        echo \"Building on macOS\"

      [windows]
      build:
        echo \"Building on Windows\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_with_multiple_os_attributes() {
    Test::new(indoc! {
      "
      [windows]
      [linux]
      build:
        echo \"Building on Linux or Windows\"

      [linux]
      build:
        echo \"Building on macOS\"

      [macos]
      build:
        echo \"Building on macOS\"
      "
    })
    .error("Duplicate recipe name `build`")
    .run();
  }

  #[test]
  fn recipe_with_conflicting_multiple_os_attributes() {
    Test::new(indoc! {
      "
      [linux]
      [openbsd]
      build:
        echo \"Building on Linux and OpenBSD\"

      [linux]
      build:
        echo \"Building on Linux again\"
      "
    })
    .error("Duplicate recipe name `build`")
    .run();
  }

  #[test]
  fn recipe_with_all_os_attributes() {
    Test::new(indoc! {
      "
      [linux]
      [windows]
      [unix]
      [macos]
      [openbsd]
      build:
        echo \"Building everywhere\"

      test:
        echo \"Testing\"
      "
    })
    .run();
  }

  #[test]
  fn circular_dependencies_self() {
    Test::new(indoc! {
      "
      foo: foo
        echo \"foo\"
      "
    })
    .error("Recipe `foo` depends on itself")
    .run();
  }

  #[test]
  fn circular_dependencies_simple() {
    Test::new(indoc! {
      "
      foo: bar
        echo \"foo\"

      bar: foo
        echo \"bar\"
      "
    })
    .error("Recipe `foo` has circular dependency `foo -> bar -> foo`")
    .error("Recipe `bar` has circular dependency `bar -> foo -> bar`")
    .run();
  }

  #[test]
  fn circular_dependencies_long_chain() {
    Test::new(indoc! {
      "
      foo: bar
        echo \"foo\"

      bar: baz
        echo \"bar\"

      baz: foo
        echo \"baz\"
      "
    })
    .error("Recipe `foo` has circular dependency `foo -> bar -> baz -> foo`")
    .error("Recipe `bar` has circular dependency `bar -> baz -> foo -> bar`")
    .error("Recipe `baz` has circular dependency `baz -> foo -> bar -> baz`")
    .run();
  }

  #[test]
  fn circular_dependencies_only_flags_cycle_members() {
    Test::new(indoc! {
      "
      foo: bar
        echo \"foo\"

      bar: baz
        echo \"bar\"

      baz: bar
        echo \"baz\"
      "
    })
    .error("Recipe `bar` has circular dependency `bar -> baz -> bar`")
    .error("Recipe `baz` has circular dependency `baz -> bar -> baz`")
    .run();
  }

  #[test]
  fn circular_dependencies_with_multiple_dependencies() {
    Test::new(indoc! {
      "
      foo: bar baz
        echo \"foo\"

      bar:
        echo \"bar\"

      baz: qux
        echo \"baz\"

      qux: foo
        echo \"qux\"
      "
    })
    .error("Recipe `foo` has circular dependency `foo -> baz -> qux -> foo`")
    .error("Recipe `baz` has circular dependency `baz -> qux -> foo -> baz`")
    .error("Recipe `qux` has circular dependency `qux -> foo -> baz -> qux`")
    .run();
  }

  #[test]
  fn circular_dependencies_multiple_cycles() {
    Test::new(indoc! {
      "
      a: b
        echo \"a\"

      b: a
        echo \"b\"

      x: y
        echo \"x\"

      y: z
        echo \"y\"

      z: x
        echo \"z\"
      "
    })
    .error("Recipe `a` has circular dependency `a -> b -> a`")
    .error("Recipe `b` has circular dependency `b -> a -> b`")
    .error("Recipe `x` has circular dependency `x -> y -> z -> x`")
    .error("Recipe `y` has circular dependency `y -> z -> x -> y`")
    .error("Recipe `z` has circular dependency `z -> x -> y -> z`")
    .run();
  }
}
