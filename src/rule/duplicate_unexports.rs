use super::*;

define_rule! {
  DuplicateUnexportRule {
    id: "duplicate-unexport",
    message: "duplicate unexport",
  }
}

impl DuplicateUnexportRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for unexport in context.duplicate_declarations(
      context.unexports(),
      |unexport| &unexport.name,
      |unexport| &unexport.attributes,
    ) {
      diagnostics.push(Diagnostic::error(
        format!(
          "Variable `{}` is unexported multiple times",
          unexport.name.value
        ),
        unexport.name.range,
      ));
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_duplicate_unexports() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "unexport foo\nunexport foo\n")
      .run();
  }

  #[test]
  fn analyzer_imported_duplicate_unexport_reports_local_declaration() {
    AnalyzerTest::new("unexport foo\nimport 'foo.just'\n")
      .file("foo.just", "\n\nunexport foo\n")
      .error(
        "Variable `foo` is unexported multiple times",
        lsp::Range::at(0, 9, 0, 12),
      )
      .run();
  }

  #[test]
  fn duplicate_unexports_are_rejected() {
    AnalyzerTest::new(indoc! {
      "
      unexport FOO
      unexport FOO
      "
    })
    .error(
      "Variable `FOO` is unexported multiple times",
      lsp::Range::at(1, 9, 1, 12),
    )
    .run();
  }

  #[test]
  fn duplicate_unexports_only_conflict_on_overlapping_platforms() {
    AnalyzerTest::new(indoc! {
      "
      [linux]
      unexport FOO
      [windows]
      unexport FOO
      [windows]
      unexport FOO
      "
    })
    .error(
      "Variable `FOO` is unexported multiple times",
      lsp::Range::at(5, 9, 5, 12),
    )
    .run();
  }
}
