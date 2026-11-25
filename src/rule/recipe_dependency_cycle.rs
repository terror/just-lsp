use super::*;

define_rule! {
  /// Detects circular dependency chains between recipes to prevent infinite
  /// execution loops.
  RecipeDependencyCycleRule {
    id: "recipe-dependency-cycle",
    title: "circular dependency",
    severity: Error,
  }
}

impl RecipeDependencyCycleRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let graph = context
      .resolved_recipes()
      .flat_map(|recipe| {
        recipe
          .dependencies
          .iter()
          .filter(|dependency| dependency.name.modules().is_empty())
          .map(move |dependency| {
            (
              recipe.name.value.as_str(),
              dependency.name.name().value.as_str(),
            )
          })
      })
      .collect::<Graph<_>>();

    context
      .resolved_recipes()
      .filter(|recipe| recipe.uri == context.document().uri)
      .filter_map(|recipe| {
        let name = &recipe.name.value;

        let cycle = graph.cycle(name.as_str())?;

        let message = if cycle.len() == 2 {
          format!("Recipe `{name}` depends on itself")
        } else {
          format!(
            "Recipe `{name}` has circular dependency `{}`",
            cycle.join(" -> ")
          )
        };

        Some(Diagnostic::new(message, recipe.range))
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::{formatdoc, indoc},
  };

  #[test]
  fn circular_dependencies_long_chain() {
    AnalyzerTest::new(indoc! {
      "
      foo: bar
        echo \"foo\"

      bar: baz
        echo \"bar\"

      baz: foo
        echo \"baz\"
      "
    })
    .error(
      "Recipe `foo` has circular dependency `foo -> bar -> baz -> foo`",
      lsp::Range::at(0, 0, 3, 0),
    )
    .error(
      "Recipe `bar` has circular dependency `bar -> baz -> foo -> bar`",
      lsp::Range::at(3, 0, 6, 0),
    )
    .error(
      "Recipe `baz` has circular dependency `baz -> foo -> bar -> baz`",
      lsp::Range::at(6, 0, 8, 0),
    )
    .run();
  }

  #[test]
  fn circular_dependencies_multiple_cycles() {
    AnalyzerTest::new(indoc! {
      "
      a: b
        echo \"a\"

      b: a
        echo \"b\"

      x: y
        echo \"x\"

      y: z
        echo \"y\"

      z: x
        echo \"z\"
      "
    })
    .error(
      "Recipe `a` has circular dependency `a -> b -> a`",
      lsp::Range::at(0, 0, 3, 0),
    )
    .error(
      "Recipe `b` has circular dependency `b -> a -> b`",
      lsp::Range::at(3, 0, 6, 0),
    )
    .error(
      "Recipe `x` has circular dependency `x -> y -> z -> x`",
      lsp::Range::at(6, 0, 9, 0),
    )
    .error(
      "Recipe `y` has circular dependency `y -> z -> x -> y`",
      lsp::Range::at(9, 0, 12, 0),
    )
    .error(
      "Recipe `z` has circular dependency `z -> x -> y -> z`",
      lsp::Range::at(12, 0, 14, 0),
    )
    .run();
  }

  #[test]
  fn circular_dependencies_only_flags_cycle_members() {
    AnalyzerTest::new(indoc! {
      "
      foo: bar
        echo \"foo\"

      bar: baz
        echo \"bar\"

      baz: bar
        echo \"baz\"
      "
    })
    .error(
      "Recipe `bar` has circular dependency `bar -> baz -> bar`",
      lsp::Range::at(3, 0, 6, 0),
    )
    .error(
      "Recipe `baz` has circular dependency `baz -> bar -> baz`",
      lsp::Range::at(6, 0, 8, 0),
    )
    .run();
  }

  #[test]
  fn circular_dependencies_overlapping_cycles() {
    #[track_caller]
    fn case(dependencies: &str, message: &'static str) {
      AnalyzerTest::new(&formatdoc! {
        "
        foo: {dependencies}
        bar: foo
        baz: foo
        "
      })
      .error(message, lsp::Range::at(0, 0, 1, 0))
      .error(
        "Recipe `bar` has circular dependency `bar -> foo -> bar`",
        lsp::Range::at(1, 0, 2, 0),
      )
      .error(
        "Recipe `baz` has circular dependency `baz -> foo -> baz`",
        lsp::Range::at(2, 0, 3, 0),
      )
      .run();
    }

    case(
      "bar baz",
      "Recipe `foo` has circular dependency `foo -> bar -> foo`",
    );

    case(
      "baz bar",
      "Recipe `foo` has circular dependency `foo -> baz -> foo`",
    );
  }

  #[test]
  fn circular_dependencies_report_only_local_members() {
    AnalyzerTest::new("foo: bar\nimport 'foo.just'\n")
      .file("foo.just", "bar: foo\n")
      .error(
        "Recipe `foo` has circular dependency `foo -> bar -> foo`",
        lsp::Range::at(0, 0, 1, 0),
      )
      .run();
  }

  #[test]
  fn circular_dependencies_report_winning_declaration() {
    AnalyzerTest::new(indoc! {
      "
      set allow-duplicate-recipes
      foo:
      foo: foo
      "
    })
    .error("Recipe `foo` depends on itself", lsp::Range::at(2, 0, 3, 0))
    .run();
  }

  #[test]
  fn circular_dependencies_self() {
    AnalyzerTest::new(indoc! {
      "
      foo: foo
        echo \"foo\"
      "
    })
    .error("Recipe `foo` depends on itself", lsp::Range::at(0, 0, 2, 0))
    .run();
  }

  #[test]
  fn circular_dependencies_shortest_return_path() {
    AnalyzerTest::new(indoc! {
      "
      foo: bar
      bar: baz foo
      baz: foo
      "
    })
    .error(
      "Recipe `foo` has circular dependency `foo -> bar -> foo`",
      lsp::Range::at(0, 0, 1, 0),
    )
    .error(
      "Recipe `bar` has circular dependency `bar -> baz -> foo -> bar`",
      lsp::Range::at(1, 0, 2, 0),
    )
    .error(
      "Recipe `baz` has circular dependency `baz -> foo -> bar -> baz`",
      lsp::Range::at(2, 0, 3, 0),
    )
    .run();
  }

  #[test]
  fn circular_dependencies_simple() {
    AnalyzerTest::new(indoc! {
      "
      foo: bar
        echo \"foo\"

      bar: foo
        echo \"bar\"
      "
    })
    .error(
      "Recipe `foo` has circular dependency `foo -> bar -> foo`",
      lsp::Range::at(0, 0, 3, 0),
    )
    .error(
      "Recipe `bar` has circular dependency `bar -> foo -> bar`",
      lsp::Range::at(3, 0, 5, 0),
    )
    .run();
  }

  #[test]
  fn circular_dependencies_use_root_dependencies() {
    AnalyzerTest::new(indoc! {
      "
      set allow-duplicate-recipes
      foo: bar
      bar: foo
      import 'foo.just'
      "
    })
    .file("foo.just", "bar:\n")
    .error(
      "Recipe `foo` has circular dependency `foo -> bar -> foo`",
      lsp::Range::at(1, 0, 2, 0),
    )
    .error(
      "Recipe `bar` has circular dependency `bar -> foo -> bar`",
      lsp::Range::at(2, 0, 3, 0),
    )
    .run();
  }

  #[test]
  fn circular_dependencies_with_multiple_dependencies() {
    AnalyzerTest::new(indoc! {
      "
      foo: bar baz
        echo \"foo\"

      bar:
        echo \"bar\"

      baz: qux
        echo \"baz\"

      qux: foo
        echo \"qux\"
      "
    })
    .error(
      "Recipe `foo` has circular dependency `foo -> baz -> qux -> foo`",
      lsp::Range::at(0, 0, 3, 0),
    )
    .error(
      "Recipe `baz` has circular dependency `baz -> qux -> foo -> baz`",
      lsp::Range::at(6, 0, 9, 0),
    )
    .error(
      "Recipe `qux` has circular dependency `qux -> foo -> baz -> qux`",
      lsp::Range::at(9, 0, 11, 0),
    )
    .run();
  }
}
