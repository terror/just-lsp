use super::*;

define_rule! {
  MissingModuleFileRule {
    id: "missing-module-file",
    title: "missing module file",
    severity: Error,
  }
}

impl MissingModuleFileRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    context
      .view()
      .dependencies()
      .filter_map(|dependency| {
        Some(Diagnostic::new(
          format!(
            "Could not find source file for module `{}`",
            dependency.missing_module()?
          ),
          dependency.location,
        ))
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn module_paths() {
    fn case(justfile: &str) -> AnalyzerTest {
      AnalyzerTest::new(justfile).file("foo.just", "bar baz:\n")
    }

    case("mod foo\n\nqux: (foo::bar 'a')\n").run();

    case("mod foo\n\nalias qux := foo::bar\n").run();

    case("mod foo\n\nalias qux := foo\n").run();

    case("mod foo\n\nqux: foo::bar\n")
      .error(
        "Dependency `foo::bar` requires 1 argument, but 0 provided",
        lsp::Range::at(2, 5, 2, 13),
      )
      .run();

    case("mod foo\n\nqux: foo::baz\n")
      .error(
        "Recipe `foo::baz` not found. Did you mean `foo::bar`?",
        lsp::Range::at(2, 5, 2, 13),
      )
      .run();

    case("mod foo\n\nalias qux := bob::bar\n")
      .error("Module `bob` not found", lsp::Range::at(2, 13, 2, 16))
      .run();

    case("mod? bob\n\nqux: bob::bar\n").run();

    case("mod bob\n\nqux: bob::bar\n")
      .error(
        "Could not find source file for module `bob`",
        lsp::Range::at(0, 4, 0, 7),
      )
      .run();

    case("mod foo 'bob.just'\n")
      .error(
        "Could not find source file for module `foo`",
        lsp::Range::at(0, 8, 0, 18),
      )
      .run();
  }
}
