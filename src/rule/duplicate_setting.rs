use super::*;

define_rule! {
  /// Emits diagnostics when the same `set` option is declared more than once.
  DuplicateSettingRule {
    id: "duplicate-setting",
    title: "duplicate setting",
    severity: Error,
  }
}

impl DuplicateSettingRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for setting in context.duplicate_declarations(
      context.module().settings(),
      |setting| &setting.name,
      |setting| &setting.attributes,
    ) {
      diagnostics.push(Diagnostic::new(
        format!("Duplicate setting `{}`", setting.name.value),
        setting.range,
      ));
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_duplicate_settings() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "set export\nset export\n")
      .run();
  }

  #[test]
  fn analyzer_imported_duplicate_setting_reports_local_declaration() {
    AnalyzerTest::new("set export\nimport 'foo.just'\n")
      .file("foo.just", "\n\nset export\n")
      .error("Duplicate setting `export`", lsp::Range::at(0, 0, 1, 0))
      .run();
  }

  #[test]
  fn settings_dotenv_command_duplicate() {
    AnalyzerTest::new(indoc! {
      "
      set dotenv-command := \"foo\"
      set dotenv-command := \"bar\"
      "
    })
    .error(
      "Duplicate setting `dotenv-command`",
      lsp::Range::at(1, 0, 2, 0),
    )
    .run();
  }

  #[test]
  fn settings_duplicate() {
    AnalyzerTest::new(indoc! {
      "
      set export := true
      set shell := [\"bash\", \"-c\"]
      set export := false

      foo:
        echo \"foo\"
      "
    })
    .error("Duplicate setting `export`", lsp::Range::at(2, 0, 3, 0))
    .run();
  }

  #[test]
  fn settings_with_platform_attributes() {
    AnalyzerTest::new(indoc! {
      "
      [unix]
      set shell := ['sh', '-cu']

      [windows]
      set shell := ['cmd', '/c']
      "
    })
    .run();
  }
}
