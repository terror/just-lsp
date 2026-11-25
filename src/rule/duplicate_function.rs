use super::*;

define_rule! {
  /// Flags user-defined functions that reuse the same name multiple times.
  DuplicateFunctionRule {
    id: "duplicate-function",
    title: "duplicate function",
    severity: Error,
  }
}

impl DuplicateFunctionRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    context
      .duplicate_declarations(
        context.functions(),
        |function| &function.name,
        |function| &function.attributes,
      )
      .into_iter()
      .map(|function| {
        Diagnostic::new(
          format!("Duplicate function `{}`", function.name.value),
          function.range,
        )
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_duplicate_functions() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "foo() := 'bar'\nfoo() := 'baz'\n")
      .run();
  }

  #[test]
  fn analyzer_imported_duplicate_function_reports_local_declaration() {
    AnalyzerTest::new("set unstable\n_foo() := 'bar'\nimport 'foo.just'\n")
      .file("foo.just", "\n\n_foo() := 'baz'\n")
      .error("Duplicate function `_foo`", lsp::Range::at(1, 0, 2, 0))
      .run();
  }

  #[test]
  fn analyzer_later_function_declaration_wins() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      foo() := 'bar'
      foo(bar) := bar
      baz:
        echo {{ foo('bar') }}
      "
    })
    .error("Duplicate function `foo`", lsp::Range::at(2, 0, 3, 0))
    .run();
  }

  #[test]
  fn user_defined_function_duplicates() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      foo() := \"bar\"
      foo() := \"baz\"
      foo() := \"bat\"

      bar:
        echo {{foo()}}
      "
    })
    .error("Duplicate function `foo`", lsp::Range::at(3, 0, 4, 0))
    .error("Duplicate function `foo`", lsp::Range::at(4, 0, 5, 0))
    .run();
  }

  #[test]
  fn user_defined_function_duplicates_platform_specific() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      [unix]
      foo() := \"foo\"

      [windows]
      foo() := \"bar\"

      bar:
        echo {{foo()}}
      "
    })
    .run();
  }
}
