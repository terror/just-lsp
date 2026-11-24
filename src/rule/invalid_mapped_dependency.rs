use super::*;

define_rule! {
  InvalidMappedDependencyRule {
    id: "invalid-mapped-dependency",
    message: "invalid mapped dependency",
    run(context) {
      let mut diagnostics = Vec::new();

      let lists = context.setting_enabled("lists");

      for recipe in context.local_declarations(context.recipes()) {
        for dependency in &recipe.dependencies {
          let starred = dependency
            .arguments
            .iter()
            .filter_map(|argument| argument.starred)
            .collect::<Vec<_>>();

          match dependency.mapped {
            Some(range) => {
              if !lists {
                diagnostics.push(Diagnostic::error(
                  "Mapped dependencies require `set lists`".to_string(),
                  range,
                ));
              }

              if starred.is_empty() {
                diagnostics.push(Diagnostic::error(
                  "Mapped dependencies must include a starred argument".to_string(),
                  range,
                ));
              }

              for range in starred.iter().skip(1) {
                diagnostics.push(Diagnostic::error(
                  "Mapped dependencies may not include multiple starred arguments".to_string(),
                  *range,
                ));
              }
            }
            None => {
              for range in starred {
                diagnostics.push(Diagnostic::error(
                  "Starred dependency arguments require mapped dependencies".to_string(),
                  range,
                ));
              }
            }
          }
        }
      }

      diagnostics
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_mapped_dependencies() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "foo: *(bar 'baz')\n")
      .run();
  }

  #[test]
  fn recipe_dependencies_mapped() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      bar arg:
        echo {{arg}}

      foo *args: *(bar *args)
        echo {{args}}
      "
    })
    .run();
  }

  #[test]
  fn recipe_dependencies_mapped_reject_multiple_starred_arguments() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      bar arg other:
        echo {{arg}} {{other}}

      foo *args: *(bar *args *args)
        echo {{args}}
      "
    })
    .error(
      "Mapped dependencies may not include multiple starred arguments",
      lsp::Range::at(6, 23, 6, 24),
    )
    .run();
  }

  #[test]
  fn recipe_dependencies_mapped_require_lists() {
    AnalyzerTest::new(indoc! {
      "
      bar arg:
        echo {{arg}}

      foo *args: *(bar *args)
        echo {{args}}
      "
    })
    .error(
      "Mapped dependencies require `set lists`",
      lsp::Range::at(3, 11, 3, 12),
    )
    .run();
  }

  #[test]
  fn recipe_dependencies_mapped_require_starred_argument() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      bar arg:
        echo {{arg}}

      foo args: *(bar args)
        echo {{args}}
      "
    })
    .error(
      "Mapped dependencies must include a starred argument",
      lsp::Range::at(6, 10, 6, 11),
    )
    .run();
  }

  #[test]
  fn recipe_dependencies_starred_arguments_require_mapped_dependency() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      bar arg:
        echo {{arg}}

      foo *args: (bar *args)
        echo {{args}}
      "
    })
    .error(
      "Starred dependency arguments require mapped dependencies",
      lsp::Range::at(6, 16, 6, 17),
    )
    .run();
  }
}
