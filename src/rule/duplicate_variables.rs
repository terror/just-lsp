use super::*;

define_rule! {
  /// Emits diagnostics when variable assignments reuse the same name without
  /// explicitly opting into overriding via `allow-duplicate-variables`.
  DuplicateVariableRule {
    id: "duplicate-variable",
    title: "duplicate variable",
    severity: Error,
  }
}

impl DuplicateVariableRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let allow_duplicates = context
      .module()
      .setting_enabled("allow-duplicate-variables");

    if allow_duplicates {
      return Vec::new();
    }

    let mut diagnostics = Vec::new();

    for variable in context.duplicate_declarations(
      context.module().variables(),
      |variable| &variable.name,
      |variable| &variable.attributes,
    ) {
      diagnostics.push(Diagnostic::new(
        format!("Duplicate variable `{}`", variable.name.value),
        variable.range,
      ));
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_duplicate_variables() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "foo := 'bar'\nfoo := 'baz'\n")
      .run();
  }

  #[test]
  fn analyzer_imported_duplicate_variable_reports_local_declaration() {
    AnalyzerTest::new("export foo := 'bar'\nimport 'foo.just'\n")
      .file("foo.just", "\n\nexport foo := 'baz'\n")
      .error("Duplicate variable `foo`", lsp::Range::at(0, 7, 1, 0))
      .run();
  }

  #[test]
  fn duplicate_variable_assignments() {
    AnalyzerTest::new(indoc! {
      "
      foo := \"one\"
      foo := \"two\"

      recipe:
        echo {{ foo }}
      "
    })
    .error("Duplicate variable `foo`", lsp::Range::at(1, 0, 2, 0))
    .run();
  }

  #[test]
  fn duplicate_variable_assignments_allowed_via_setting() {
    AnalyzerTest::new(indoc! {
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
  fn duplicate_variable_assignments_platform_specific() {
    AnalyzerTest::new(indoc! {
      "
      [unix]
      foo := \"foo\"

      [windows]
      foo := \"bar\"

      bar:
        echo {{ foo }}
      "
    })
    .run();
  }
}
