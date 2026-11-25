use super::*;

define_rule! {
  /// Reports recipe dependencies that reference recipes which don't exist in
  /// the current document.
  UnresolvedDependencyRule {
    id: "unresolved-dependency",
    title: "missing dependency",
    severity: Error,
  }
}

impl UnresolvedDependencyRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    context
      .local()
      .recipes()
      .flat_map(|recipe| &recipe.dependencies)
      .filter_map(|dependency| {
        let path = &dependency.name;

        Some(match context.module().unresolved_recipe(path)? {
          UnresolvedRecipe::Module(segment) => Diagnostic::new(
            format!("Module `{}` not found", segment.value),
            segment.range,
          ),
          UnresolvedRecipe::Recipe { suggestion: None } => Diagnostic::new(
            format!("Recipe `{path}` not found"),
            dependency.range,
          ),
          UnresolvedRecipe::Recipe {
            suggestion: Some(suggestion),
          } => Diagnostic::new(
            format!("Recipe `{path}` not found. Did you mean `{suggestion}`?"),
            dependency.range,
          )
          .quickfix(Quickfix::replacement(
            path.name(),
            suggestion.name().value.clone(),
          )),
        })
      })
      .collect()
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
