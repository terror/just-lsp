use super::*;

define_rule! {
  /// Warns when recipe lines use indentation that differs from the first recipe
  /// line, matching the behavior of the `just` parser.
  InconsistentIndentationRule {
    id: "inconsistent-recipe-indentation",
    message: "inconsistent indentation",
  }
}

impl InconsistentIndentationRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let default_script = context.setting_enabled("default-script");

    context
      .local_declarations(context.recipes())
      .filter_map(|recipe| {
        let script = recipe.runs_as_script(default_script);

        let mut lines = recipe
          .body_lines()
          .filter(|line| script || line.kind != IndentKind::Mixed);

        let expected =
          lines.next().filter(|line| line.kind != IndentKind::Mixed)?;

        lines
          .scan(expected.continues, |continues, line| {
            Some((mem::replace(continues, line.continues), line))
          })
          .find(|(previous_continues, line)| {
            if script {
              !line.indent.starts_with(expected.indent)
                && !line.mixed_prefix(expected.indent.len())
            } else {
              !*previous_continues
                && line.kind == expected.kind
                && line.indent != expected.indent
            }
          })
          .map(|(_, line)| {
            Diagnostic::error(
              format!(
                "Recipe line has inconsistent leading whitespace. \
                 Recipe started with `{}` but found line with `{}`",
                Self::visualize_whitespace(expected.indent),
                Self::visualize_whitespace(line.indent),
              ),
              line.range(),
            )
          })
      })
      .collect()
  }

  fn visualize_whitespace(indent: &str) -> String {
    if indent.is_empty() {
      return "∅".to_string();
    }

    indent
      .chars()
      .map(|ch| match ch {
        ' ' => '␠',
        '\t' => '⇥',
        other => other,
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_inconsistent_indentation() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "foo:\n  echo foo\n    echo bar\n")
      .run();
  }

  #[test]
  fn default_script_recipe_is_exempt_from_inconsistent_indentation() {
    AnalyzerTest::new(indoc! {
      "
      set default-script

      build-docs:
        if true; then
          echo docs
        fi
      "
    })
    .run();
  }

  #[test]
  fn recipe_consistent_indentation() {
    AnalyzerTest::new("foo:\n  echo \"foo\"\n  echo \"bar\"\n").run();
  }

  #[test]
  fn recipe_inconsistent_indentation_after_continuation() {
    AnalyzerTest::new("foo:\n  bar \\\n    baz\n    qux\n      quux\n")
      .error(
        "Recipe line has inconsistent leading whitespace. Recipe started with `␠␠` but found line with `␠␠␠␠`",
        lsp::Range::at(3, 0, 3, 4),
      )
      .run();
  }

  #[test]
  fn recipe_inconsistent_indentation_after_multiline_interpolation() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        {{'
      bar
          baz
      '}}
          qux
      "
    })
    .error(
      "Recipe line has inconsistent leading whitespace. Recipe started with `␠␠` but found line with `␠␠␠␠`",
      lsp::Range::at(5, 0, 5, 4),
    )
    .run();
  }

  #[test]
  fn recipe_inconsistent_indentation_between_lines() {
    AnalyzerTest::new(indoc! {
      "
      foo:
              echo foo
        echo bar
      "
    })
    .error(
      "Recipe line has inconsistent leading whitespace. Recipe started with `␠␠␠␠␠␠␠␠` but found line with `␠␠`",
      lsp::Range::at(2, 0, 2, 2),
    )
    .run();
  }

  #[test]
  fn recipe_inconsistent_indentation_shebang_comment() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        bar
          #!/bin/sh
        qux
      "
    })
    .error(
      "Recipe line has inconsistent leading whitespace. Recipe started with `␠␠` but found line with `␠␠␠␠`",
      lsp::Range::at(2, 0, 2, 4),
    )
    .run();
  }

  #[test]
  fn recipe_inconsistent_indentation_with_attribute() {
    AnalyzerTest::new(indoc! {
      "
      [private]
      foo:
              echo foo
        echo bar
      "
    })
    .error(
      "Recipe line has inconsistent leading whitespace. Recipe started with `␠␠␠␠␠␠␠␠` but found line with `␠␠`",
      lsp::Range::at(3, 0, 3, 2),
    )
    .run();
  }

  #[test]
  fn recipe_inconsistent_indentation_with_multiline_header() {
    AnalyzerTest::new(indoc! {
      "
      foo bar='''
      \tbaz
          qux
      ''':
              echo {{bar}}
        echo foo
      "
    })
    .error(
      "Recipe line has inconsistent leading whitespace. Recipe started with `␠␠␠␠␠␠␠␠` but found line with `␠␠`",
      lsp::Range::at(5, 0, 5, 2),
    )
    .run();
  }

  #[test]
  fn recipe_inconsistent_indentation_with_multiple_attributes() {
    AnalyzerTest::new(indoc! {
      "
      bar:

      [private]
      [no-cd]
      foo:
              echo foo
        echo bar
      "
    })
    .error(
      "Recipe line has inconsistent leading whitespace. Recipe started with `␠␠␠␠␠␠␠␠` but found line with `␠␠`",
      lsp::Range::at(6, 0, 6, 2),
    )
    .run();
  }

  #[test]
  fn recipe_inconsistent_indentation_with_shell_attribute() {
    AnalyzerTest::new(indoc! {
      "
      set default-script
      [shell]
      foo:
              echo foo
        echo bar
      "
    })
    .error(
      "Recipe line has inconsistent leading whitespace. Recipe started with `␠␠␠␠␠␠␠␠` but found line with `␠␠`",
      lsp::Range::at(4, 0, 4, 2),
    )
    .run();
  }

  #[test]
  fn recipe_inconsistent_indentation_with_shell_shebang() {
    AnalyzerTest::new(indoc! {
      "
      [shell]
      foo:
              #!/bin/sh
        echo foo
      "
    })
    .error(
      "Recipe line has inconsistent leading whitespace. Recipe started with `␠␠␠␠␠␠␠␠` but found line with `␠␠`",
      lsp::Range::at(3, 0, 3, 2),
    )
    .run();
  }

  #[test]
  fn recipe_indentation_multiline_interpolation() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        {{'
      bar
          baz
      '}}
      "
    })
    .run();
  }

  #[test]
  fn recipe_indentation_multiline_interpolation_continuation() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        {{'
      bar
          baz
      '}} \\
          qux
        quux
      "
    })
    .run();
  }

  #[test]
  fn recipe_indentation_multiline_interpolation_tabs() {
    AnalyzerTest::new(indoc! {
      "
      [private]
      foo:
        {{'
      bar
      \tbaz
      '}}
      "
    })
    .run();
  }

  #[test]
  fn recipe_indentation_script_attribute() {
    AnalyzerTest::new(indoc! {
      "
      [script]
      foo:
        bar
          baz
        \tbaz
      \tqux
      "
    })
    .error(
      "Recipe line has inconsistent leading whitespace. Recipe started with `␠␠` but found line with `⇥`",
      lsp::Range::at(5, 0, 5, 1),
    )
    .run();
  }

  #[test]
  fn recipe_line_continuations_allow_extra_indentation() {
    AnalyzerTest::new(indoc! {
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
    AnalyzerTest::new(indoc! {
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
}
