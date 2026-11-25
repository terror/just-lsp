use super::*;

define_rule! {
  DotenvCommandConflictRule {
    id: "dotenv-command-conflict",
    title: "conflicting dotenv command setting",
    severity: Error,
  }
}

impl DotenvCommandConflictRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    context
      .conflicting_settings(|previous, current| {
        previous.name.value == "dotenv-command" && current.loads_dotenv()
          || current.name.value == "dotenv-command" && previous.loads_dotenv()
      })
      .into_iter()
      .map(|(previous, current)| {
        Diagnostic::new(
          format!(
            "`{}` is incompatible with `{}`",
            previous.name.value, current.name.value,
          ),
          current.range,
        )
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_dotenv_command_conflicts() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "set dotenv-command := 'foo'\nset dotenv-load\n")
      .run();
  }

  #[test]
  fn analyzer_imported_dotenv_command_conflict_reports_local_load() {
    AnalyzerTest::new("set dotenv-load\nimport 'foo.just'\n")
      .file("foo.just", "\n\nset dotenv-command := 'foo'\n")
      .error(
        "`dotenv-command` is incompatible with `dotenv-load`",
        lsp::Range::at(0, 0, 1, 0),
      )
      .run();
  }

  #[test]
  fn analyzer_imported_dotenv_load_conflict_reports_local_command() {
    AnalyzerTest::new("set dotenv-command := 'foo'\nimport 'foo.just'\n")
      .file("foo.just", "\n\nset dotenv-load\n")
      .error(
        "`dotenv-load` is incompatible with `dotenv-command`",
        lsp::Range::at(0, 0, 1, 0),
      )
      .run();
  }

  #[test]
  fn disjoint_dotenv_settings_do_not_conflict() {
    AnalyzerTest::new(indoc! {
      "
      [linux]
      set dotenv-command := 'foo'

      [windows]
      set dotenv-path := 'bar'
      "
    })
    .run();
  }

  #[test]
  fn dotenv_command_allows_disabled_loading_and_override() {
    AnalyzerTest::new(indoc! {
      "
      set dotenv-command := 'foo'
      set dotenv-load := false
      set dotenv-override
      "
    })
    .run();
  }

  #[test]
  fn dotenv_command_conflict_deduplicates_identical_diagnostics() {
    AnalyzerTest::new(indoc! {
      "
      [linux]
      set dotenv-command := 'foo'

      [windows]
      set dotenv-command := 'bar'

      set dotenv-path := 'baz'
      "
    })
    .error(
      "`dotenv-command` is incompatible with `dotenv-path`",
      lsp::Range::at(6, 0, 7, 0),
    )
    .run();
  }

  #[test]
  fn dotenv_command_conflict_preserves_distinct_locations() {
    AnalyzerTest::new(indoc! {
      "
      set dotenv-command := 'foo'

      [linux]
      set dotenv-path := 'bar'

      [windows]
      set dotenv-path := 'baz'
      "
    })
    .error(
      "`dotenv-command` is incompatible with `dotenv-path`",
      lsp::Range::at(2, 0, 4, 0),
    )
    .error(
      "`dotenv-command` is incompatible with `dotenv-path`",
      lsp::Range::at(5, 0, 7, 0),
    )
    .run();
  }

  #[test]
  fn dotenv_command_conflicts_with_loading_settings() {
    #[track_caller]
    fn case(justfile: &str, message: &'static str) {
      AnalyzerTest::new(justfile)
        .error(message, lsp::Range::at(1, 0, 2, 0))
        .run();
    }

    case(
      "set dotenv-command := 'foo'\nset dotenv-filename := 'bar'\n",
      "`dotenv-command` is incompatible with `dotenv-filename`",
    );

    case(
      "set dotenv-command := 'foo'\nset dotenv-load\n",
      "`dotenv-command` is incompatible with `dotenv-load`",
    );

    case(
      "set dotenv-required\nset dotenv-command := 'foo'\n",
      "`dotenv-required` is incompatible with `dotenv-command`",
    );
  }
}
