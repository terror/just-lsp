use super::*;

define_rule! {
  MissingModuleFileRule {
    id: "missing-module-file",
    message: "missing module file",
    run(context) {
      context
        .view()
        .dependencies()
        .filter_map(|dependency| {
          Some(Diagnostic::error(
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
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn module_paths() {
    #[track_caller]
    fn case(justfile: &str, expected: &[(&str, lsp::Range)]) {
      let module =
        TestModule::new(&[("justfile", justfile), ("foo.just", "bar baz:\n")]);

      let analyzer = Analyzer {
        config: None,
        view: module.view(),
      };

      assert_eq!(
        analyzer
          .analyze()
          .iter()
          .map(|diagnostic| (diagnostic.message.as_str(), diagnostic.range))
          .collect::<Vec<_>>(),
        expected,
      );
    }

    case("mod foo\n\nqux: (foo::bar 'a')\n", &[]);

    case("mod foo\n\nalias qux := foo::bar\n", &[]);

    case("mod foo\n\nalias qux := foo\n", &[]);

    case(
      "mod foo\n\nqux: foo::bar\n",
      &[(
        "Dependency `foo::bar` requires 1 argument, but 0 provided",
        lsp::Range::at(2, 5, 2, 13),
      )],
    );

    case(
      "mod foo\n\nqux: foo::baz\n",
      &[(
        "Recipe `foo::baz` not found. Did you mean `foo::bar`?",
        lsp::Range::at(2, 5, 2, 13),
      )],
    );

    case(
      "mod foo\n\nalias qux := bob::bar\n",
      &[("Module `bob` not found", lsp::Range::at(2, 13, 2, 16))],
    );

    case("mod? bob\n\nqux: bob::bar\n", &[]);

    case(
      "mod bob\n\nqux: bob::bar\n",
      &[(
        "Could not find source file for module `bob`",
        lsp::Range::at(0, 4, 0, 7),
      )],
    );

    case(
      "mod foo 'bob.just'\n",
      &[(
        "Could not find source file for module `foo`",
        lsp::Range::at(0, 8, 0, 18),
      )],
    );
  }
}
