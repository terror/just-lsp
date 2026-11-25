use super::*;

define_rule! {
  /// Validates recipe parameter lists for duplicate names, ordering mistakes,
  /// and illegal variadic/default combinations.
  DuplicateRecipeParameterRule {
    id: "duplicate-recipe-parameter",
    message: "invalid recipe parameters",
  }
}

impl DuplicateRecipeParameterRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for recipe in context.local_declarations(context.recipes()) {
      let mut seen = HashSet::new();

      for param in &recipe.parameters {
        if !seen.insert(param.name.clone()) {
          diagnostics.push(Diagnostic::error(
            format!("Duplicate parameter `{}`", param.name),
            param.range,
          ));
        }
      }
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_duplicate_recipe_parameters() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "foo bar bar:\n")
      .run();
  }

  #[test]
  fn recipe_parameters_duplicate() {
    AnalyzerTest::new(indoc! {
      "
      recipe_with_duplicate_param arg1 arg1:
        echo \"{{arg1}}\"
      "
    })
    .error("Duplicate parameter `arg1`", lsp::Range::at(0, 33, 0, 37))
    .run();
  }
}
