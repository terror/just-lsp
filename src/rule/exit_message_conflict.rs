use super::*;

define_rule! {
  ExitMessageConflictRule {
    id: "exit-message-conflict",
    title: "conflicting exit message attributes",
    severity: Error,
  }
}

impl ExitMessageConflictRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    context
      .conflicting_recipe_attributes("exit-message", "no-exit-message")
      .into_iter()
      .map(|(recipe, attribute)| {
        Diagnostic::new(
          format!(
            "Recipe `{}` can't combine `[exit-message]` with `[no-exit-message]`",
            recipe.name.value,
          ),
          attribute.range,
        )
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_exit_message_conflicts() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "[exit-message]\n[no-exit-message]\nfoo:\n")
      .run();
  }

  #[test]
  fn attributes_exit_message_conflicts_with_no_exit_message() {
    AnalyzerTest::new(indoc! {
      "
      [exit-message]
      [no-exit-message]
      build:
        echo \"build\"
      "
    })
    .error(
      "Recipe `build` can't combine `[exit-message]` with `[no-exit-message]`",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }
}
