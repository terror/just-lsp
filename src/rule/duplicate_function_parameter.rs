use super::*;

define_rule! {
  /// Reports invalid user-defined function parameter lists.
  DuplicateFunctionParameterRule {
    id: "duplicate-function-parameter",
    title: "invalid function parameters",
    severity: Error,
  }
}

impl DuplicateFunctionParameterRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    context
      .local_declarations(context.functions())
      .flat_map(|function| {
        let mut seen = HashSet::new();

        function
          .parameters
          .iter()
          .filter(move |parameter| !seen.insert(parameter.value.clone()))
          .map(|parameter| {
            Diagnostic::new(
              format!("Duplicate parameter `{}`", parameter.value),
              parameter.range,
            )
          })
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_function_parameters() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "foo(bar, bar) := bar\n")
      .run();
  }

  #[test]
  fn user_defined_function_duplicate_parameters() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      foo(bar, bar) := bar

      bar:
        echo {{foo('bar', 'baz')}}
      "
    })
    .error("Duplicate parameter `bar`", lsp::Range::at(2, 9, 2, 12))
    .run();
  }
}
