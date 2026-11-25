use super::*;

define_rule! {
  /// Ensures every function call references a builtin function recognized by
  /// `just`.
  UnknownFunctionRule {
    id: "unknown-function",
    title: "unknown function",
    severity: Error,
  }
}

impl UnknownFunctionRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for function_call in context.local().function_calls() {
      let function_name = &function_call.name.value;

      if Builtin::function(function_name.as_str()).is_none()
        && context.module().find_function(function_name).is_none()
      {
        let suggestion = function_name.find_suggestion(
          Builtin::functions().flat_map(Builtin::names).chain(
            context
              .module()
              .resolved_functions()
              .map(|function| function.name.value.as_str()),
          ),
        );

        let message = match &suggestion {
          Some(suggestion) => format!(
            "Unknown function `{function_name}`. Did you mean `{suggestion}`?"
          ),
          None => format!("Unknown function `{function_name}`"),
        };

        let quickfix = suggestion.map(|suggestion| {
          Quickfix::replacement(&function_call.name, suggestion)
        });

        diagnostics.push(
          Diagnostic::new(message, function_call.name.range).quickfix(quickfix),
        );
      }
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn dir_aliases_recognized() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo {{ home_dir() }}
        echo {{ cache_dir() }}
        echo {{ config_dir() }}
        echo {{ config_local_dir() }}
        echo {{ data_dir() }}
        echo {{ data_local_dir() }}
        echo {{ executable_dir() }}
        echo {{ invocation_dir() }}
        echo {{ invocation_dir_native() }}
        echo {{ justfile_dir() }}
        echo {{ source_dir() }}
        echo {{ parent_dir('~/.config') }}
      "
    })
    .run();
  }

  #[test]
  fn function_calls_recipe_name_recognized() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo {{ recipe_name() }}
      "
    })
    .run();
  }

  #[test]
  fn function_calls_unknown() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo {{ unknown_function() }}
      "
    })
    .error(
      "Unknown function `unknown_function`",
      lsp::Range::at(1, 10, 1, 26),
    )
    .run();
  }

  #[test]
  fn user_defined_function_not_flagged_as_unknown() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      foo(x) := x + \"!\"

      bar:
        echo {{ foo(\"baz\") }}
      "
    })
    .run();
  }

  #[test]
  fn user_defined_function_shadows_builtin() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      env_var() := 'ok'
      x := env_var()

      foo:
        echo {{x}}
      "
    })
    .run();
  }
}
