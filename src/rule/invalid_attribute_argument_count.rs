use super::*;

define_rule! {
  /// Reports attribute invocations whose argument counts don't match their
  /// builtin definitions.
  InvalidAttributeArgumentCountRule {
    id: "invalid-attribute-argument-count",
    message: "invalid attribute arguments",
    run(context) {
      let mut diagnostics = Vec::new();

      for attribute in context.attributes() {
        let attribute_name = &attribute.name.value;

        let Some(Builtin::Attribute { signature, .. }) =
          context.builtin_attribute(attribute_name)
        else {
          continue;
        };

        let argument_count = attribute.positional_arguments().count();

        let range = signature.positional.argument_range();

        if range.contains(&argument_count) {
          continue;
        }

        let (min, max) = (*range.start(), *range.end());

        let expected = match max {
          usize::MAX => format!("at least {min}"),
          _ if min == max => format!("{min}"),
          _ => format!("{min}-{max}"),
        };

        diagnostics.push(Diagnostic::error(
          format!(
            "Attribute `{attribute_name}` got {argument_count} {} but takes {expected} {}",
            Count("argument", argument_count),
            if min == 1 && matches!(max, 1 | usize::MAX) { "argument" } else { "arguments" },
          ),
          attribute.range,
        ));
      }

      diagnostics
    }
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::{formatdoc, indoc},
  };

  #[test]
  fn arg_attribute_empty_parens() {
    AnalyzerTest::new(indoc! {
      "
      [arg()]
      bar foo:
        echo {{foo}}
      "
    })
    .error(
      "Attribute `arg` got 0 arguments but takes 1 argument",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn arg_attribute_extra_positional_argument() {
    AnalyzerTest::new(indoc! {
      "
      [arg('foo', 'bar')]
      bar foo:
        echo {{foo}}
      "
    })
    .error(
      "Attribute `arg` got 2 arguments but takes 1 argument",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn arg_attribute_missing_parameter_name() {
    #[track_caller]
    fn case(attribute: &str) {
      AnalyzerTest::new(&formatdoc! {
        "
        {attribute}
        bar foo:
          echo {{{{foo}}}}
        "
      })
      .error(
        "Attribute `arg` got 0 arguments but takes 1 argument",
        lsp::Range::at(0, 0, 1, 0),
      )
      .run();
    }

    case("[arg]");
    case("[arg(long)]");
    case("[arg(long='foo')]");
  }

  #[test]
  fn attribute_arguments() {
    #[track_caller]
    fn case(attribute: &str) {
      AnalyzerTest::new(&formatdoc! {
        "
        {attribute}
        foo bar:
          echo {{{{bar}}}}
        "
      })
      .run();
    }

    case("[private()]");
    case("[doc()]");
    case("[continue()]");
    case("[doc('foo',)]");
    case("[arg('bar', short,)]");
  }

  #[test]
  fn attributes_extra_arguments() {
    AnalyzerTest::new(indoc! {
      "
      [linux('invalid')]
      foo:
        echo \"foo\"
      "
    })
    .error(
      "Attribute `linux` got 1 argument but takes 0 arguments",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();

    AnalyzerTest::new(indoc! {
      "
      [default('invalid')]
      foo:
        echo \"foo\"
      "
    })
    .error(
      "Attribute `default` got 1 argument but takes 0 arguments",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn attributes_inline_parameters_focused() {
    AnalyzerTest::new(indoc! {
      "
      [group: 'foo', no-cd]
      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_metadata_multiple_arguments() {
    AnalyzerTest::new(indoc! {
      "
      [metadata('foo', 'bar')]
      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_missing_arguments() {
    AnalyzerTest::new(indoc! {
      "
      [extension]
      foo:
        #!/usr/bin/env bash
        echo \"foo\"
      "
    })
    .error(
      "Attribute `extension` got 0 arguments but takes 1 argument",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn attributes_more_arguments_than_required() {
    AnalyzerTest::new(indoc! {
      "
      [group('foo', 'bar')]
      foo:
        echo \"foo\"
      "
    })
    .error(
      "Attribute `group` got 2 arguments but takes 1 argument",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn attributes_no_parameters_needed() {
    AnalyzerTest::new(indoc! {
      "
      [script]
      foo:
        echo \"foo\"
      "
    })
    .run();

    AnalyzerTest::new(indoc! {
      "
      [confirm]
      foo:
        echo \"foo\"
      "
    })
    .run();

    AnalyzerTest::new(indoc! {
      "
      [default]
      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_optional_extra_arguments() {
    AnalyzerTest::new(indoc! {
      "
      [confirm('foo', 'bar')]
      foo:
      "
    })
    .error(
      "Attribute `confirm` got 2 arguments but takes 0-1 arguments",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn cache_attribute_rejects_positional_arguments() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      [script]
      [cache('foo')]
      build:
        cargo build
      "
    })
    .error(
      "Attribute `cache` got 1 argument but takes 0 arguments",
      lsp::Range::at(3, 0, 4, 0),
    )
    .run();
  }

  #[test]
  fn env_attribute_missing_value() {
    AnalyzerTest::new(indoc! {
      "
      [env('FOO')]
      foo:
        echo \"$FOO\"
      "
    })
    .error(
      "Attribute `env` got 1 argument but takes 2 arguments",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn env_attribute_multiple_vars_allowed() {
    AnalyzerTest::new(indoc! {
      "
      [env('FOO', 'bar')]
      [env('BAZ', 'qux')]
      foo:
        echo \"$FOO $BAZ\"
      "
    })
    .run();
  }

  #[test]
  fn env_attribute_shorthand_reports_argument_count() {
    AnalyzerTest::new(indoc! {
      "
      foo := 'bar'

      [env: foo]
      recipe:
        echo foo
      "
    })
    .error(
      "Attribute `env` got 1 argument but takes 2 arguments",
      lsp::Range::at(2, 0, 3, 0),
    )
    .run();
  }

  #[test]
  fn env_attribute_too_many_arguments() {
    AnalyzerTest::new(indoc! {
      "
      [env('FOO', 'bar', 'extra')]
      foo:
        echo \"$FOO\"
      "
    })
    .error(
      "Attribute `env` got 3 arguments but takes 2 arguments",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn env_attribute_valid() {
    AnalyzerTest::new(indoc! {
      "
      [env('FOO', 'bar')]
      foo:
        echo \"$FOO\"
      "
    })
    .run();
  }
}
