use super::*;

define_rule! {
  ScriptShellConflictRule {
    id: "script-shell-conflict",
    message: "conflicting script attributes",
    run(context) {
      context
        .conflicting_recipe_attributes("script", "shell")
        .into_iter()
        .map(|(recipe, attribute)| {
          Diagnostic::error(
            format!(
              "Recipe `{}` can't combine `[script]` with `[shell]`",
              recipe.name.value,
            ),
            attribute.range,
          )
        })
        .collect()
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_script_shell_conflicts() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "[script]\n[shell]\nfoo:\n")
      .run();
  }

  #[test]
  fn cache_with_script_and_shell() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      [script]
      [shell]
      [cache]
      build:
        cargo build
      "
    })
    .error(
      "Recipe `build` can't combine `[script]` with `[shell]`",
      lsp::Range::at(2, 0, 3, 0),
    )
    .run();
  }
}
