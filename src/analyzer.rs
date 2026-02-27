use super::*;

#[derive(Debug)]
pub(crate) struct Analyzer<'a> {
  document: &'a Document,
}

impl<'a> Analyzer<'a> {
  /// Analyzes the document and returns a list of diagnostics.
  pub(crate) fn analyze(&self) -> Vec<Diagnostic> {
    let context = RuleContext::new(self.document);

    let mut diagnostics = inventory::iter::<&dyn Rule>
      .into_iter()
      .flat_map(|rule| {
        rule
          .run(&context)
          .into_iter()
          .map(move |diagnostic| Diagnostic {
            id: rule.id().to_string(),
            display: rule.message().to_string(),
            ..diagnostic
          })
      })
      .collect::<Vec<_>>();

    diagnostics.sort_by(|a, b| {
      a.range
        .start
        .line
        .cmp(&b.range.start.line)
        .then_with(|| a.range.start.character.cmp(&b.range.start.character))
        .then_with(|| a.message.cmp(&b.message))
    });

    diagnostics
  }

  /// Creates a new analyzer for the given document.
  #[must_use]
  pub(crate) fn new(document: &'a Document) -> Self {
    Self { document }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, Message::*, indoc::indoc, pretty_assertions::assert_eq};

  type RangeSpec = (u32, u32, u32, u32);

  fn to_lsp_range(
    (start_line, start_character, end_line, end_character): RangeSpec,
  ) -> lsp::Range {
    lsp::Range {
      start: lsp::Position {
        line: start_line,
        character: start_character,
      },
      end: lsp::Position {
        line: end_line,
        character: end_character,
      },
    }
  }

  #[derive(Debug)]
  enum Message<'a> {
    Scoped { text: &'a str, range: RangeSpec },
    Text(&'a str),
  }

  #[derive(Debug)]
  struct Test {
    document: Document,
    messages: Vec<(Message<'static>, Option<lsp::DiagnosticSeverity>)>,
  }

  impl Test {
    fn error(self, message: Message<'static>) -> Self {
      Self {
        messages: self
          .messages
          .into_iter()
          .chain([(message, Some(lsp::DiagnosticSeverity::ERROR))])
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
      let Test { document, messages } = self;

      let analyzer = Analyzer::new(&document);

      let diagnostics = analyzer
        .analyze()
        .into_iter()
        .map(lsp::Diagnostic::from)
        .collect::<Vec<lsp::Diagnostic>>();

      assert_eq!(
        diagnostics.len(),
        messages.len(),
        "Expected diagnostics {:?} but got {:?}",
        messages,
        diagnostics,
      );

      for (diagnostic, (expected_message, expected_severity)) in
        diagnostics.into_iter().zip(messages.into_iter())
      {
        assert_eq!(diagnostic.severity, expected_severity, "{diagnostic:?}");

        match expected_message {
          Text(expected) => assert_eq!(diagnostic.message, *expected),
          Scoped { text, range } => {
            assert_eq!(diagnostic.message, *text);
            assert_eq!(diagnostic.range, to_lsp_range(range));
          }
        }
      }
    }

    fn warning(self, message: Message<'static>) -> Self {
      Self {
        messages: self
          .messages
          .into_iter()
          .chain([(message, Some(lsp::DiagnosticSeverity::WARNING))])
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
    .error(Message::Scoped {
      text: "Duplicate alias `bar`",
      range: (4, 0, 4, 16),
    })
    .error(Message::Scoped {
      text: "Duplicate alias `bar`",
      range: (5, 0, 5, 16),
    })
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
    .error(Message::Text("Recipe `baz` not found"))
    .run();
  }

  #[test]
  fn aliases_missing_target() {
    Test::new(indoc! {
      "
      alias foo :=
      "
    })
    .error(Message::Text("Missing identifier in alias"))
    .error(Message::Text("Recipe `` not found"))
    .run();
  }

  #[test]
  fn parallel_without_dependencies_warns() {
    Test::new(indoc! {
      "
      [parallel]
      foo:
        echo \"foo\"
      "
    })
    .warning(Message::Text(
      "Recipe `foo` has no dependencies, so `[parallel]` has no effect",
    ))
    .run();
  }

  #[test]
  fn parallel_with_single_dependency_warns() {
    Test::new(indoc! {
      "
      [parallel]
      foo: bar
        echo \"foo\"

      bar:
        echo \"bar\"
      "
    })
    .warning(Message::Text(
      "Recipe `foo` has only one dependency, so `[parallel]` has no effect",
    ))
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
    .error(Message::Text("Recipe `t` is redefined as an alias"))
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
    .error(Message::Text("Alias `t` is redefined as a recipe"))
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
    .error(Message::Text("Recipe `missing` not found"))
    .error(Message::Text("Recipe `nonexistent` not found"))
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
    .error(Message::Text(
      "Recipe `ci` has duplicate `[default]` attribute, which may only appear once per module"),
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
    .error(Message::Text(
      "Recipe `build` has duplicate `[default]` attribute, which may only appear once per module"),
    )
    .run();
  }

  #[test]
  fn attributes_duplicate_recipe_attribute() {
    Test::new(indoc! {
      "
      [script]
      [script]
      build:
        echo \"build\"
      "
    })
    .error(Message::Text("Recipe attribute `script` is duplicated"))
    .run();
  }

  #[test]
  fn attributes_duplicate_working_directory_attribute() {
    Test::new(indoc! {
      "
      [working-directory: 'foo']
      [working-directory: 'bar']
      build:
        echo \"build\"
      "
    })
    .error(Message::Text(
      "Recipe attribute `working-directory` is duplicated",
    ))
    .run();
  }

  #[test]
  fn attributes_working_directory_conflicts_with_no_cd() {
    Test::new(indoc! {
      "
      [no-cd]
      [working-directory: '/tmp']
      build:
        echo \"build\"
      "
    })
    .error(Message::Text(
      "Recipe `build` can't combine `[working-directory]` with `[no-cd]`",
    ))
    .run();
  }

  #[test]
  fn attributes_no_cd_allowed_with_global_working_directory() {
    Test::new(indoc! {
      "
      set working-directory := '/tmp'

      [no-cd]
      build:
        echo \"build\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_multiple_group_attributes_allowed() {
    Test::new(indoc! {
      "
      [group('lint')]
      [group('rust')]
      build:
        echo \"build\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_duplicate_group_attribute() {
    Test::new(indoc! {
      "
      [group('dev')]
      [group('dev')]
      build:
        echo \"build\"
      "
    })
    .error(Message::Text("Recipe attribute `group` is duplicated"))
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
    .error(Message::Text(
      "Attribute `linux` got 1 argument but takes 0 arguments",
    ))
    .run();

    Test::new(indoc! {
      "
      [default('invalid')]
      foo:
        echo \"foo\"
      "
    })
    .error(Message::Text(
      "Attribute `default` got 1 argument but takes 0 arguments",
    ))
    .run();
  }

  #[test]
  fn attributes_missing_arguments() {
    Test::new(indoc! {
      "
      [extension]
      foo:
        echo \"foo\"
      "
    })
    .error(Message::Text(
      "Attribute `extension` got 0 arguments but takes 1 argument",
    ))
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
    .error(Message::Text("Unknown attribute `unknown_attribute`"))
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
    .error(Message::Text(
      "Recipe `publish` has both shebang line and `[script]` attribute",
    ))
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
    .error(Message::Text(
      "Attribute `group` cannot be applied to alias target",
    ))
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
    .error(Message::Text("Unknown attribute `foo`"))
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
    .error(Message::Text(
      "Attribute `group` got 2 arguments but takes 1 argument",
    ))
    .run();
  }

  #[test]
  fn attributes_metadata_multiple_arguments() {
    Test::new(indoc! {
      "
      [metadata('foo', 'bar')]
      foo:
        echo \"foo\"
      "
    })
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
    .error(Message::Text(
      "Function `replace` requires at least 3 arguments, but 0 provided",
    ))
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
    .error(Message::Text(
      "Function `uppercase` accepts 1 argument, but 2 provided",
    ))
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
    .error(Message::Text("Unknown function `unknown_function`"))
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
    .error(Message::Text("Syntax error near `foo echo \"foo\"`"))
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
    .error(Message::Text(
      "Recipe `foo` mixes tabs and spaces for indentation",
    ))
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
    .error(Message::Text(
      "Recipe `foo` mixes tabs and spaces for indentation",
    ))
    .run();
  }

  #[test]
  fn recipe_inconsistent_indentation_between_lines() {
    Test::new("foo:\n        echo \"foo\"\n  echo \"bar\"\n")
    .error(Message::Text(
      "Recipe line has inconsistent leading whitespace. Recipe started with `␠␠␠␠␠␠␠␠` but found line with `␠␠`"),
    )
    .run();
  }

  #[test]
  fn recipe_consistent_indentation() {
    Test::new("foo:\n  echo \"foo\"\n  echo \"bar\"\n").run();
  }

  #[test]
  fn recipe_line_continuations_allow_extra_indentation() {
    Test::new(indoc! {
      "
      update-mdbook-theme:
        curl \\
          https://example.com/resource \\
          > docs/theme/index.hbs
      "
    })
    .run();
  }

  #[test]
  fn shebang_recipe_is_exempt_from_inconsistent_indentation() {
    Test::new(indoc! {
      "
      build-docs:
        #!/usr/bin/env bash
        mdbook build docs -d build
        for language in ar de; do
          echo $language
        done
      "
    })
    .run();
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
  fn parser_errors_valid_with_shell_expanded_strings() {
    Test::new(indoc! {
      r#"
      import x'~/.config/just/common.just'

      greeting := x"~/$USER/${GREETING:-hello}"

      foo:
        echo {{greeting}}
      "#
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
    .error(Message::Text("Recipe `baz` not found"))
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
    .error(Message::Text("Recipe `missing1` not found"))
    .error(Message::Text("Recipe `missing2` not found"))
    .run();
  }

  #[test]
  fn recipe_dependencies_duplicate_warns() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo foo
        echo \"bar\"
      "
    })
    .warning(Message::Text(
      "Recipe `bar` lists dependency `foo` more than once; just only runs it once, so it's redundant",
    ))
    .run();
  }

  #[test]
  fn recipe_dependencies_duplicate_with_arguments_warns() {
    Test::new(indoc! {
      "
      foo arg1:
        echo \"{{arg1}}\"

      bar: (foo `a`) (foo `a`)
        echo \"bar\"
      "
    })
    .warning(Message::Text(
      "Recipe `bar` lists dependency `foo` with the same arguments more than once; just only runs it once, so it's redundant",
    ))
    .run();
  }

  #[test]
  fn recipe_dependencies_with_different_arguments_no_warning() {
    Test::new(indoc! {
      "
      foo arg1:
        echo \"{{arg1}}\"

      bar: (foo `a`) (foo `b`)
        echo \"bar\"
      "
    })
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
    .error(Message::Text(
      "Dependency `foo` requires 2 arguments, but 0 provided",
    ))
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
    .error(Message::Text(
      "Dependency `foo` requires 2 arguments, but 1 provided",
    ))
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
    .error(Message::Text(
      "Dependency `foo` accepts 1 argument, but 3 provided",
    ))
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
    .error(Message::Text("Variable `wow` not found"))
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
    .error(Message::Text("Duplicate parameter `arg1`"))
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
    .error(Message::Text(
      "Required parameter `arg2` follows a parameter with a default value",
    ))
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
    .error(Message::Text("Setting `export` expects a boolean value"))
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
    .error(Message::Text("Duplicate setting `export`"))
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
    .error(Message::Text("Unknown setting `unknown-setting`"))
    .error(Message::Text("Duplicate setting `export`"))
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
  fn settings_string_type_correct_with_shell_expanded_string() {
    Test::new(indoc! {
      r#"
      set dotenv-path := x"~/.env.${JUST_ENV:-development}"

      foo:
        echo "foo"
      "#
    })
    .run();
  }

  #[test]
  fn settings_shell_array_accepts_shell_expanded_strings() {
    Test::new(indoc! {
      r#"
      set shell := [x"${SHELL_BIN:-bash}", x"-c"]

      foo:
        echo "foo"
      "#
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
    .error(Message::Text(
      "Setting `dotenv-path` expects a string value",
    ))
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
    .error(Message::Text("Unknown setting `unknown-setting`"))
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
    .error(Message::Text("Variable `var` not found"))
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
    .warning(Message::Text("Parameter `bar` appears unused"))
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
    .warning(Message::Text("Parameter `bar` appears unused"))
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
  fn positional_arguments_setting_marks_parameters_as_used() {
    Test::new(indoc! {
      "
      set positional-arguments := true

      graph log:
        ./bin/graph $1
      "
    })
    .run();
  }

  #[test]
  fn positional_arguments_attribute_marks_parameters_as_used() {
    Test::new(indoc! {
      "
      [positional-arguments]
      graph log:
        ./bin/graph $1
      "
    })
    .run();
  }

  #[test]
  fn positional_arguments_dollar_at_marks_all_as_used() {
    Test::new(indoc! {
      r#"
      [positional-arguments]
      run *args:
        #!/usr/bin/env bash
        exec "$@"
      "#
    })
    .run();
  }

  #[test]
  fn positional_arguments_disabled_still_warns() {
    Test::new(indoc! {
      "
      graph log:
        ./bin/graph $1
      "
    })
    .warning(Message::Text("Parameter `log` appears unused"))
    .run();
  }

  #[test]
  fn positional_arguments_only_mark_used_indices() {
    Test::new(indoc! {
      "
      set positional-arguments := true

      graph first second:
        ./bin/graph $2
      "
    })
    .warning(Message::Text("Parameter `first` appears unused"))
    .run();
  }

  #[test]
  fn positional_arguments_setting_handles_multiple_parameters() {
    Test::new(indoc! {
      "
      set positional-arguments := true

      graph first second third:
        ./bin/graph $1 ${2} $3
      "
    })
    .run();
  }

  #[test]
  fn positional_arguments_setting_handles_multiple_parameters_unused() {
    Test::new(indoc! {
      "
      set positional-arguments := true

      graph first second third fourth:
        ./bin/graph $1 ${2} $3
      "
    })
    .warning(Message::Scoped {
      text: "Parameter `fourth` appears unused",
      range: (2, 25, 2, 31),
    })
    .run();
  }

  #[test]
  fn positional_arguments_attribute_scope_is_limited() {
    Test::new(indoc! {
      "
      [positional-arguments]
      graph log:
        ./bin/graph $1

      other data:
        ./bin/graph $1
      "
    })
    .warning(Message::Text("Parameter `data` appears unused"))
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
    .error(Message::Text("Duplicate recipe name `foo`"))
    .error(Message::Text("Duplicate recipe name `foo`"))
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
    .warning(Message::Text("Variable `foo` appears unused"))
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
    .error(Message::Text("Duplicate variable `foo`"))
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
    .warning(Message::Text("Variable `unused` appears unused"))
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
    .error(Message::Text("Variable `foo` not found"))
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
    .warning(Message::Text("Variable `unused_var` appears unused"))
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
    .warning(Message::Text("Variable `param` appears unused"))
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
    .warning(Message::Text("Variable `never_used` appears unused"))
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
    .warning(Message::Text("Variable `foo` appears unused"))
    .run();
  }

  #[test]
  fn unexported_variables_warned() {
    Test::new(indoc! {
      "
      foo := \"unused value\"
      unexport BAR := \"unexported but unused\"
      baz := \"used value\"

      recipe:
        echo {{ baz }}
      "
    })
    .warning(Message::Text("Variable `foo` appears unused"))
    .warning(Message::Text("Variable `BAR` appears unused"))
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
    .error(Message::Text("Duplicate recipe name `build`"))
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
    .error(Message::Text("Duplicate recipe name `build`"))
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
    .error(Message::Text("Duplicate recipe name `build`"))
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
    .error(Message::Text("Duplicate recipe name `build`"))
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
    .error(Message::Text("Duplicate recipe name `build`"))
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
    .error(Message::Text("Duplicate recipe name `build`"))
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
    .error(Message::Text("Duplicate recipe name `build`"))
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
  fn comma_separated_os_attributes_no_conflict() {
    Test::new(indoc! {
      "
      [private, unix]
      hello:
        @echo 'hello'

      [private, windows]
      hello:
        @echo hello
      "
    })
    .run();
  }

  #[test]
  fn comma_separated_os_attributes_with_conflict() {
    Test::new(indoc! {
      "
      [private, linux]
      hello:
        @echo 'hello on linux'

      [private, linux]
      hello:
        @echo 'hello on linux again'
      "
    })
    .error(Message::Text("Duplicate recipe name `hello`"))
    .run();
  }

  #[test]
  fn comma_separated_unix_windows_no_conflict() {
    Test::new(indoc! {
      "
      [unix]
      [private]
      build:
        @echo 'building on unix'

      [private, windows]
      build:
        @echo 'building on windows'
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
    .error(Message::Text("Recipe `foo` depends on itself"))
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
    .error(Message::Text(
      "Recipe `foo` has circular dependency `foo -> bar -> foo`",
    ))
    .error(Message::Text(
      "Recipe `bar` has circular dependency `bar -> foo -> bar`",
    ))
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
    .error(Message::Text(
      "Recipe `foo` has circular dependency `foo -> bar -> baz -> foo`",
    ))
    .error(Message::Text(
      "Recipe `bar` has circular dependency `bar -> baz -> foo -> bar`",
    ))
    .error(Message::Text(
      "Recipe `baz` has circular dependency `baz -> foo -> bar -> baz`",
    ))
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
    .error(Message::Text(
      "Recipe `bar` has circular dependency `bar -> baz -> bar`",
    ))
    .error(Message::Text(
      "Recipe `baz` has circular dependency `baz -> bar -> baz`",
    ))
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
    .error(Message::Text(
      "Recipe `foo` has circular dependency `foo -> baz -> qux -> foo`",
    ))
    .error(Message::Text(
      "Recipe `baz` has circular dependency `baz -> qux -> foo -> baz`",
    ))
    .error(Message::Text(
      "Recipe `qux` has circular dependency `qux -> foo -> baz -> qux`",
    ))
    .run();
  }

  #[test]
  fn arg_attribute_valid() {
    Test::new(indoc! {
      "
      [arg('foo', help=\"Help text\")]
      bar foo:
        echo {{foo}}
      "
    })
    .run();
  }

  #[test]
  fn arg_attribute_with_long_option() {
    Test::new(indoc! {
      "
      [arg('foo', long=\"foo-opt\")]
      bar foo:
        echo {{foo}}
      "
    })
    .run();
  }

  #[test]
  fn arg_attribute_with_short_option() {
    Test::new(indoc! {
      "
      [arg('foo', short=\"f\")]
      bar foo:
        echo {{foo}}
      "
    })
    .run();
  }

  #[test]
  fn arg_attribute_with_pattern() {
    Test::new(indoc! {
      "
      [arg('version', pattern=\"[0-9]+\\\\.[0-9]+\\\\.[0-9]+\")]
      release version:
        echo {{version}}
      "
    })
    .run();
  }

  #[test]
  fn arg_attribute_with_multiple_options() {
    Test::new(indoc! {
      "
      [arg('foo', long=\"foo-opt\", short=\"f\", value=\"default\")]
      bar foo:
        echo {{foo}}
      "
    })
    .run();
  }

  #[test]
  fn arg_attribute_missing_parameter_name() {
    Test::new(indoc! {
      "
      [arg]
      bar foo:
        echo {{foo}}
      "
    })
    .error(Message::Text(
      "Attribute `arg` got 0 arguments but takes at least 1 argument",
    ))
    .run();
  }

  #[test]
  fn arg_attribute_empty_parens() {
    Test::new(indoc! {
      "
      [arg()]
      bar foo:
        echo {{foo}}
      "
    })
    .error(Message::Text(
      "Attribute `arg` got 0 arguments but takes at least 1 argument",
    ))
    .error(Message::Text("Missing identifier in attribute named param"))
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
    .error(Message::Text(
      "Recipe `a` has circular dependency `a -> b -> a`",
    ))
    .error(Message::Text(
      "Recipe `b` has circular dependency `b -> a -> b`",
    ))
    .error(Message::Text(
      "Recipe `x` has circular dependency `x -> y -> z -> x`",
    ))
    .error(Message::Text(
      "Recipe `y` has circular dependency `y -> z -> x -> y`",
    ))
    .error(Message::Text(
      "Recipe `z` has circular dependency `z -> x -> y -> z`",
    ))
    .run();
  }

  #[test]
  fn format_strings_with_valid_variables() {
    Test::new(indoc! {
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
  fn format_strings_with_undefined_variables() {
    Test::new(indoc! {
      r"
      greeting := f'Hello, {{undefined_var}}!'
      foo:
        echo {{greeting}}
      "
    })
    .error(Message::Text("Variable `undefined_var` not found"))
    .run();
  }

  #[test]
  fn format_strings_with_function_calls() {
    Test::new(indoc! {
      r"
      info := f'arch: {{arch()}}'
      foo:
        echo {{info}}
      "
    })
    .run();
  }

  #[test]
  fn format_strings_mark_variables_as_used() {
    Test::new(indoc! {
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
  fn module_path_dependency_not_flagged() {
    Test::new(indoc! {"
      foo: tools::build
        echo foo
    "})
    .run();
  }

  #[test]
  fn module_path_alias_not_flagged() {
    Test::new(indoc! {"
      alias b := tools::build
    "})
    .run();
  }

  #[test]
  fn recipe_named_import() {
    Test::new(indoc! {
      r"
      run: import

      import:
        body
      "
    })
    .run();
  }
}
