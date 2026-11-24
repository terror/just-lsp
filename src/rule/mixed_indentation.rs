use super::*;

define_rule! {
  /// Detects recipes that mix tabs and spaces for indentation, which often
  /// results in confusing or invalid `just` bodies.
  MixedIndentationRule {
    id: "mixed-recipe-indentation",
    message: "mixed indentation",
    run(context) {
      let default_script = context.setting_enabled("default-script");

      context
        .local_declarations(context.recipes())
        .filter_map(|recipe| {
          let script = recipe.runs_as_script(default_script);

          recipe
            .body_lines()
            .try_fold(None, |expected: Option<RecipeLine>, line| {
              let mixed = match &expected {
                None => line.kind == IndentKind::Mixed,
                Some(expected) if script => {
                  !line.indent.starts_with(expected.indent)
                    && line.mixed_prefix(expected.indent.len())
                }
                Some(expected) => {
                  line.kind == IndentKind::Mixed || expected.kind != line.kind
                }
              };

              if mixed {
                ControlFlow::Break(Diagnostic::error(
                  format!(
                    "Recipe `{}` mixes tabs and spaces for indentation",
                    recipe.name.value,
                  ),
                  line.range(),
                ))
              } else {
                ControlFlow::Continue(Some(expected.unwrap_or(line)))
              }
            })
            .break_value()
        })
        .collect()
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_mixed_indentation() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "foo:\n \techo foo\n")
      .run();
  }

  #[test]
  fn recipe_indentation_script_attribute_mixed() {
    AnalyzerTest::new(indoc! {
      "
      [script]
      foo:
        bar
       \tbaz
      "
    })
    .error(
      "Recipe `foo` mixes tabs and spaces for indentation",
      lsp::Range::at(3, 0, 3, 2),
    )
    .run();

    AnalyzerTest::new(indoc! {
      "
      [script]
      foo:
       \tbar
        baz
      "
    })
    .error(
      "Recipe `foo` mixes tabs and spaces for indentation",
      lsp::Range::at(2, 0, 2, 2),
    )
    .run();
  }

  #[test]
  fn recipe_mixed_indentation_after_multiline_interpolation() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        {{'
      bar
      \tbaz
      '}}
      \tqux
      "
    })
    .error(
      "Recipe `foo` mixes tabs and spaces for indentation",
      lsp::Range::at(5, 0, 5, 1),
    )
    .run();
  }

  #[test]
  fn recipe_mixed_indentation_between_lines() {
    AnalyzerTest::new(indoc! {
      "
      foo:
      \techo foo
        echo bar
      "
    })
    .error(
      "Recipe `foo` mixes tabs and spaces for indentation",
      lsp::Range::at(2, 0, 2, 2),
    )
    .run();
  }

  #[test]
  fn recipe_mixed_indentation_continuation_after_interpolation() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        {{'bar'}}\\
      \t{{'baz'}}
      "
    })
    .error(
      "Recipe `foo` mixes tabs and spaces for indentation",
      lsp::Range::at(2, 0, 2, 1),
    )
    .run();
  }

  #[test]
  fn recipe_mixed_indentation_continuation_after_multiline_interpolation() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        {{'
      bar
          baz
      '}}\\
      \t{{'qux'}}
      "
    })
    .error(
      "Recipe `foo` mixes tabs and spaces for indentation",
      lsp::Range::at(5, 0, 5, 1),
    )
    .run();
  }

  #[test]
  fn recipe_mixed_indentation_shebang_comment() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        bar
      \t#!/bin/sh
        qux
      "
    })
    .error(
      "Recipe `foo` mixes tabs and spaces for indentation",
      lsp::Range::at(2, 0, 2, 1),
    )
    .run();
  }

  #[test]
  fn recipe_mixed_indentation_single_line_mix() {
    AnalyzerTest::new(indoc! {
      "
      foo:
      \t  echo foo
      "
    })
    .error(
      "Recipe `foo` mixes tabs and spaces for indentation",
      lsp::Range::at(1, 0, 1, 3),
    )
    .run();
  }

  #[test]
  fn recipe_mixed_indentation_single_line_mix_with_attribute() {
    AnalyzerTest::new(indoc! {
      "
      [private]
      foo:
      \t  echo foo
      "
    })
    .error(
      "Recipe `foo` mixes tabs and spaces for indentation",
      lsp::Range::at(2, 0, 2, 3),
    )
    .run();
  }

  #[test]
  fn recipe_mixed_indentation_with_attribute() {
    AnalyzerTest::new(indoc! {
      "
      [private]
      foo:
      \techo foo
        echo bar
      "
    })
    .error(
      "Recipe `foo` mixes tabs and spaces for indentation",
      lsp::Range::at(3, 0, 3, 2),
    )
    .run();
  }

  #[test]
  fn recipe_mixed_indentation_with_multiline_header() {
    AnalyzerTest::new(indoc! {
      "
      foo bar='''
        baz
          qux
      ''':
      \techo {{bar}}
        echo foo
      "
    })
    .error(
      "Recipe `foo` mixes tabs and spaces for indentation",
      lsp::Range::at(5, 0, 5, 2),
    )
    .run();
  }

  #[test]
  fn recipe_mixed_indentation_with_multiple_attributes() {
    AnalyzerTest::new(indoc! {
      "
      bar:

      [private]
      [no-cd]
      foo:
      \techo foo
        echo bar
      "
    })
    .error(
      "Recipe `foo` mixes tabs and spaces for indentation",
      lsp::Range::at(6, 0, 6, 2),
    )
    .run();
  }

  #[test]
  fn recipe_mixed_indentation_with_shell_attribute() {
    AnalyzerTest::new(indoc! {
      "
      set default-script
      [shell]
      foo:
      \techo foo
        echo bar
      "
    })
    .error(
      "Recipe `foo` mixes tabs and spaces for indentation",
      lsp::Range::at(4, 0, 4, 2),
    )
    .run();
  }

  #[test]
  fn recipe_mixed_indentation_with_shell_shebang() {
    AnalyzerTest::new(indoc! {
      "
      [shell]
      foo:
      \t#!/bin/sh
        echo foo
      "
    })
    .error(
      "Recipe `foo` mixes tabs and spaces for indentation",
      lsp::Range::at(3, 0, 3, 2),
    )
    .run();
  }
}
