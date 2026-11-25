use super::*;

#[derive(Hash, Eq, PartialEq)]
struct DependencyArgumentKey {
  starred: bool,
  value: String,
}

#[derive(Hash, Eq, PartialEq)]
struct DependencyKey {
  arguments: Vec<DependencyArgumentKey>,
  mapped: bool,
  name: String,
  phase: DependencyPhase,
}

define_rule! {
  /// Warns when a recipe lists the same dependency with identical arguments
  /// more than once since `just` only runs it once.
  DuplicateDependencyRule {
    id: "duplicate-dependency",
    title: "duplicate dependencies",
    severity: Warning,
  }
}

impl DuplicateDependencyRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for recipe in context.local().recipes() {
      let mut seen: HashSet<DependencyKey> = HashSet::new();

      for dependency in &recipe.dependencies {
        let key = DependencyKey {
          name: dependency.name.to_string(),
          mapped: dependency.mapped.is_some(),
          phase: dependency.phase,
          arguments: dependency
            .arguments
            .iter()
            .map(|argument| DependencyArgumentKey {
              starred: argument.starred.is_some(),
              value: argument.value.clone(),
            })
            .collect(),
        };

        if !seen.insert(key) {
          let message = if dependency.arguments.is_empty() {
            format!(
              "Recipe `{}` lists dependency `{}` more than once; just only runs it once, so it's redundant",
              recipe.name.value, dependency.name
            )
          } else {
            format!(
              "Recipe `{}` lists dependency `{}` with the same arguments more than once; just only runs it once, so it's redundant",
              recipe.name.value, dependency.name
            )
          };

          diagnostics.push(Diagnostic::new(message, dependency.range));
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
  fn analyzer_ignores_imported_duplicate_dependencies() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "foo: bar bar\n")
      .run();
  }

  #[test]
  fn recipe_dependencies_duplicate_subsequents_warns() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo && foo foo
        echo \"bar\"
      "
    })
    .warning(
      "Recipe `bar` lists dependency `foo` more than once; just only runs it once, so it's redundant",
    lsp::Range::at(3, 16, 3, 19))
    .run();
  }

  #[test]
  fn recipe_dependencies_duplicate_warns() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo foo
        echo \"bar\"
      "
    })
    .warning(
      "Recipe `bar` lists dependency `foo` more than once; just only runs it once, so it's redundant",
    lsp::Range::at(3, 9, 3, 12))
    .run();
  }

  #[test]
  fn recipe_dependencies_duplicate_with_arguments_warns() {
    AnalyzerTest::new(indoc! {
      "
      foo arg1:
        echo \"{{arg1}}\"

      bar: (foo `a`) (foo `a`)
        echo \"bar\"
      "
    })
    .warning(
      "Recipe `bar` lists dependency `foo` with the same arguments more than once; just only runs it once, so it's redundant",
    lsp::Range::at(3, 15, 3, 24))
    .run();
  }

  #[test]
  fn recipe_dependencies_mapped_and_unmapped_not_duplicate() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      foo arg:
        echo {{arg}}

      bar *args: (foo args) *(foo *args)
        echo {{args}}
      "
    })
    .run();
  }

  #[test]
  fn recipe_dependencies_prior_and_subsequent_not_duplicate() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo && foo
        echo \"bar\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_dependencies_with_different_arguments_no_warning() {
    AnalyzerTest::new(indoc! {
      "
      foo arg1:
        echo \"{{arg1}}\"

      bar: (foo `a`) (foo `b`)
        echo \"bar\"
      "
    })
    .run();
  }
}
