use super::*;

define_rule! {
  /// Reports recipe dependencies that reference recipes which don't exist in
  /// the current document.
  UnresolvedDependencyRule {
    id: "unresolved-dependency",
    message: "missing dependency",
    run(context) {
      context
        .local_declarations(context.recipes())
        .flat_map(|recipe| &recipe.dependencies)
        .filter_map(|dependency| {
          context.unresolved_recipe(&dependency.name, dependency.range)
        })
        .collect()
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn recipe_dependencies_correct() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_dependencies_missing() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: baz
        echo \"bar\"
      "
    })
    .error(
      "Recipe `baz` not found. Did you mean `bar`?",
      lsp::Range::at(3, 5, 3, 8),
    )
    .run();
  }

  #[test]
  fn recipe_dependencies_multiple_missing() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: missing1 missing2
        echo \"bar\"
      "
    })
    .error("Recipe `missing1` not found", lsp::Range::at(3, 5, 3, 13))
    .error("Recipe `missing2` not found", lsp::Range::at(3, 14, 3, 22))
    .run();
  }
}
