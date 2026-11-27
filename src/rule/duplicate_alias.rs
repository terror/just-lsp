use super::*;

define_rule! {
  /// Flags alias declarations that reuse the same name multiple times.
  DuplicateAliasRule {
    id: "duplicate-alias",
    title: "duplicate alias",
    severity: Error,
  }
}

impl DuplicateAliasRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for alias in context.duplicate_declarations(context.aliases(Scope::Module))
    {
      diagnostics.push(Diagnostic::new(
        format!("Duplicate alias `{}`", alias.name.value),
        alias.range,
      ));
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn aliases_duplicate() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo \"foo\"

      alias bar := foo
      alias bar := foo
      alias bar := foo
      "
    })
    .error("Duplicate alias `bar`", lsp::Range::at(4, 0, 4, 16))
    .error("Duplicate alias `bar`", lsp::Range::at(5, 0, 5, 16))
    .run();
  }

  #[test]
  fn aliases_platform_specific() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo \"foo\"

      [unix]
      alias bar := foo

      [windows]
      alias bar := foo
      "
    })
    .run();
  }

  #[test]
  fn analyzer_ignores_imported_duplicate_aliases() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "alias foo := bar\nalias foo := bar\n")
      .run();
  }

  #[test]
  fn analyzer_imported_duplicate_alias_reports_local_declaration() {
    AnalyzerTest::new("alias foo := bar\nbar:\nimport 'foo.just'\n")
      .file("foo.just", "\n\nalias foo := bar\n")
      .error("Duplicate alias `foo`", lsp::Range::at(0, 0, 0, 16))
      .run();
  }
}
