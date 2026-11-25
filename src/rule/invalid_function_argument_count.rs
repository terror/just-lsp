use super::*;

define_rule! {
  /// Verifies function calls use a valid argument count and respect variadic
  /// constraints.
  InvalidFunctionArgumentCountRule {
    id: "invalid-function-argument-count",
    title: "invalid function arguments",
    severity: Error,
  }
}

impl InvalidFunctionArgumentCountRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for function_call in context.local().function_calls() {
      let function_name = &function_call.name.value;

      if let Some(function) = context.module().find_function(function_name) {
        let (argument_count, parameter_count) =
          (function_call.arguments.len(), function.parameters.len());

        if argument_count != parameter_count {
          diagnostics.push(Diagnostic::new(
            format!(
              "Function `{function_name}` accepts {parameter_count} {}, but {argument_count} provided",
              Count("argument", parameter_count)
            ),
            function_call.range,
          ));
        }
      } else if let Some(Builtin::Function { signature, .. }) =
        Builtin::function(function_name.as_str())
      {
        let range = signature.argument_range();

        let (min, max) = (*range.start(), *range.end());

        let argument_count = function_call.arguments.len();

        if argument_count < min {
          diagnostics.push(Diagnostic::new(
            format!(
              "Function `{function_name}` requires at least {min} {}, but {argument_count} provided",
              Count("argument", min)
            ),
            function_call.range,
          ));
        } else if argument_count > max {
          let upper = if min == max {
            format!("{max} {}", Count("argument", max))
          } else {
            format!("at most {max} {}", Count("argument", max))
          };

          diagnostics.push(Diagnostic::new(
            format!(
              "Function `{function_name}` accepts {upper}, but {argument_count} provided"
            ),
            function_call.range,
          ));
        }
      }
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::{formatdoc, indoc},
  };

  #[test]
  fn function_calls_correct() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo {{ arch() }}
        echo {{ join('a', 'b') }}
        echo {{ shell('echo foo') }}
        echo {{ shell('echo $@', 'foo', 'bar', 'baz') }}
        echo {{ join(\"a\", \"b\", \"c\") }}
        echo {{ uppercase(\"foo\",) }}
      "
    })
    .run();
  }

  #[test]
  fn function_calls_join_list_accepts_optional_separator() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      foo:
        echo {{ join_list(['foo', 'bar'], ',') }}
      "
    })
    .run();
  }

  #[test]
  fn function_calls_join_list_rejects_extra_arguments() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      foo:
        echo {{ join_list(['foo', 'bar'], ',', ';') }}
      "
    })
    .error(
      "Function `join_list` accepts at most 2 arguments, but 3 provided",
      lsp::Range::at(4, 10, 4, 45),
    )
    .run();
  }

  #[test]
  fn function_calls_len() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      foo:
        echo {{ len(['foo', 'bar']) }}
      "
    })
    .run();
  }

  #[test]
  fn function_calls_nested() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo {{ replace(parent_directory('~/.config/nvim/init.lua'), '.', 'dot-') }}
      "
    })
    .run();
  }

  #[test]
  fn function_calls_split() {
    #[track_caller]
    fn case(expression: &str) {
      AnalyzerTest::new(&formatdoc! {
        "
        set unstable
        set lists

        foo := {expression}

        bar:
          echo {{{{ foo }}}}
        "
      })
      .run();
    }

    case("split(\"foo bar\")");
    case("split(\"foo,bar\", \",\")");
  }

  #[test]
  fn function_calls_split_rejects_missing_argument() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      foo := split()

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "Function `split` requires at least 1 argument, but 0 provided",
      lsp::Range::at(3, 7, 3, 14),
    )
    .run();
  }

  #[test]
  fn function_calls_split_too_many_args() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      foo := split(\"foo,bar\", \",\", \"bar\")

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "Function `split` accepts at most 2 arguments, but 3 provided",
      lsp::Range::at(3, 7, 3, 35),
    )
    .run();
  }

  #[test]
  fn function_calls_style_accepts_optional_text() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo {{ style('error') }}
        echo {{ style('warning', 'careful') }}
      "
    })
    .run();
  }

  #[test]
  fn function_calls_too_few_args() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo {{ replace() }}
      "
    })
    .error(
      "Function `replace` requires at least 3 arguments, but 0 provided",
      lsp::Range::at(1, 10, 1, 19),
    )
    .run();
  }

  #[test]
  fn function_calls_too_few_variadic_arguments() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo {{ join() }}
        echo {{ join('a') }}
        echo {{ shell() }}
      "
    })
    .error(
      "Function `join` requires at least 2 arguments, but 0 provided",
      lsp::Range::at(1, 10, 1, 16),
    )
    .error(
      "Function `join` requires at least 2 arguments, but 1 provided",
      lsp::Range::at(2, 10, 2, 19),
    )
    .error(
      "Function `shell` requires at least 1 argument, but 0 provided",
      lsp::Range::at(3, 10, 3, 17),
    )
    .run();
  }

  #[test]
  fn function_calls_too_many_args() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo {{ uppercase(\"hello\", \"extra\") }}
      "
    })
    .error(
      "Function `uppercase` accepts 1 argument, but 2 provided",
      lsp::Range::at(1, 10, 1, 37),
    )
    .run();
  }

  #[test]
  fn user_defined_function_no_params() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      foo() := \"bar\"

      baz:
        echo {{ foo() }}
      "
    })
    .run();
  }

  #[test]
  fn user_defined_function_too_few_args() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      foo(a, b) := a + b

      bar:
        echo {{ foo(\"a\") }}
      "
    })
    .error(
      "Function `foo` accepts 2 arguments, but 1 provided",
      lsp::Range::at(5, 10, 5, 18),
    )
    .run();
  }

  #[test]
  fn user_defined_function_wrong_arity() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      foo(x) := x + \"!\"

      bar:
        echo {{ foo(\"a\", \"b\") }}
      "
    })
    .error(
      "Function `foo` accepts 1 argument, but 2 provided",
      lsp::Range::at(5, 10, 5, 23),
    )
    .run();
  }
}
