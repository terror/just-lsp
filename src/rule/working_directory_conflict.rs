use super::*;

define_rule! {
  /// Detects conflicts between working-directory and no-cd directives.
  WorkingDirectoryConflictRule {
    id: "working-directory-conflict",
    title: "conflicting working directory configuration",
    severity: Error,
  }
}

impl WorkingDirectoryConflictRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    context
      .conflicting_settings(|left, right| {
        match (left.name.value.as_str(), right.name.value.as_str()) {
          ("working-directory", "no-cd") => {
            matches!(right.kind, SettingKind::Boolean(true))
          }
          ("no-cd", "working-directory") => {
            matches!(left.kind, SettingKind::Boolean(true))
          }
          _ => false,
        }
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
      .chain(
        context
          .conflicting_recipe_attributes("working-directory", "no-cd")
          .into_iter()
          .map(|(recipe, attribute)| {
            Diagnostic::new(
              format!(
                "Recipe `{}` can't combine `[working-directory]` with `[no-cd]`",
                recipe.name.value,
              ),
              attribute.range,
            )
          }),
      )
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_working_directory_attribute_conflicts() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "[working-directory: 'foo']\n[no-cd]\nfoo:\n")
      .run();
  }

  #[test]
  fn analyzer_ignores_imported_working_directory_setting_conflicts() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "set working-directory := 'foo'\nset no-cd\n")
      .run();
  }

  #[test]
  fn analyzer_imported_no_cd_conflict_reports_local_working_directory() {
    AnalyzerTest::new("set working-directory := 'foo'\nimport 'foo.just'\n")
      .file("foo.just", "\n\nset no-cd\n")
      .error(
        "`no-cd` is incompatible with `working-directory`",
        lsp::Range::at(0, 0, 1, 0),
      )
      .run();
  }

  #[test]
  fn analyzer_imported_working_directory_conflict_reports_local_no_cd() {
    AnalyzerTest::new("set no-cd\nimport 'foo.just'\n")
      .file("foo.just", "\n\nset working-directory := 'foo'\n")
      .error(
        "`working-directory` is incompatible with `no-cd`",
        lsp::Range::at(0, 0, 1, 0),
      )
      .run();
  }

  #[test]
  fn attributes_no_cd_allowed_with_global_working_directory() {
    AnalyzerTest::new(indoc! {
      "
      set working-directory := '/tmp'

      [no-cd]
      build:
        echo \"build\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_working_directory_allowed_with_global_no_cd() {
    AnalyzerTest::new(indoc! {
      "
      set no-cd

      [working-directory: '/tmp']
      build:
        echo \"build\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_working_directory_conflicts_with_no_cd() {
    AnalyzerTest::new(indoc! {
      "
      [no-cd]
      [working-directory: '/tmp']
      build:
        echo \"build\"
      "
    })
    .error(
      "Recipe `build` can't combine `[working-directory]` with `[no-cd]`",
      lsp::Range::at(1, 0, 2, 0),
    )
    .run();
  }

  #[test]
  fn settings_disjoint_directory_settings_do_not_conflict() {
    AnalyzerTest::new(indoc! {
      "
      [linux]
      set no-cd

      [windows]
      set working-directory := 'foo'
      "
    })
    .run();
  }

  #[test]
  fn settings_false_no_cd_allows_working_directory() {
    AnalyzerTest::new(indoc! {
      "
      set no-cd := false
      set working-directory := 'foo'
      "
    })
    .run();
  }

  #[test]
  fn settings_no_cd_conflicts_with_working_directory() {
    AnalyzerTest::new(indoc! {
      "
      set no-cd
      set working-directory := 'foo'
      "
    })
    .error(
      "`no-cd` is incompatible with `working-directory`",
      lsp::Range::at(1, 0, 2, 0),
    )
    .run();
  }

  #[test]
  fn settings_working_directory_conflict_deduplicates_diagnostics() {
    AnalyzerTest::new(indoc! {
      "
      [linux]
      set no-cd

      [windows]
      set no-cd

      set working-directory := 'foo'
      "
    })
    .error(
      "`no-cd` is incompatible with `working-directory`",
      lsp::Range::at(6, 0, 7, 0),
    )
    .run();
  }
}
