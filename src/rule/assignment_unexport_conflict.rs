use super::*;

define_rule! {
  AssignmentUnexportConflictRule {
    id: "assignment-unexport-conflict",
    message: "export/unexport conflict",
    run(context) {
      let mut unexports = ConflictTracker::default();

      for unexport in context.unexports() {
        unexports.record(&unexport.name, &unexport.attributes);
      }

      let mut diagnostics = Vec::new();

      for variable in context.local_declarations(context.variables()) {
        if unexports.conflicts_with(&variable.name, &variable.attributes) {
          diagnostics.push(Diagnostic::error(
            format!(
              "Variable {} is both exported and unexported",
              variable.name.value
            ),
            variable.name.range,
          ));
        }
      }

      let mut variables = ConflictTracker::default();

      for variable in context.variables() {
        if variable.uri != context.document().uri {
          variables.record(&variable.name, &variable.attributes);
        }
      }

      for unexport in context.local_declarations(context.unexports()) {
        if variables.conflicts_with(&unexport.name, &unexport.attributes) {
          diagnostics.push(Diagnostic::error(
            format!(
              "Variable {} is both exported and unexported",
              unexport.name.value
            ),
            unexport.name.range,
          ));
        }
      }

      diagnostics
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_export_unexport_conflicts() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "export foo := 'bar'\nunexport foo\n")
      .run();
  }

  #[test]
  fn analyzer_imported_unexport_conflict_reports_local_variable() {
    AnalyzerTest::new("export foo := 'bar'\nimport 'foo.just'\n")
      .file("foo.just", "\n\nunexport foo\n")
      .error(
        "Variable foo is both exported and unexported",
        lsp::Range::at(0, 7, 0, 10),
      )
      .run();
  }

  #[test]
  fn analyzer_imported_variable_conflict_reports_local_unexport() {
    AnalyzerTest::new("unexport foo\nimport 'foo.just'\n")
      .file("foo.just", "\n\nexport foo := 'bar'\n")
      .error(
        "Variable foo is both exported and unexported",
        lsp::Range::at(0, 9, 0, 12),
      )
      .run();
  }

  #[test]
  fn export_unexport_conflict_is_rejected() {
    AnalyzerTest::new(indoc! {
      "
      unexport FOO
      export FOO := \"bar\"
      "
    })
    .error(
      "Variable FOO is both exported and unexported",
      lsp::Range::at(1, 7, 1, 10),
    )
    .run();
  }

  #[test]
  fn export_unexport_do_not_conflict_across_platforms() {
    AnalyzerTest::new(indoc! {
      "
      [linux]
      unexport FOO
      [windows]
      export FOO := \"bar\"
      "
    })
    .run();
  }
}
