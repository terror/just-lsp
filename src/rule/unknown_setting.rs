use super::*;

define_rule! {
  /// Emits diagnostics for `set` directives targeting settings that don't exist
  /// in the builtin catalog.
  UnknownSettingRule {
    id: "unknown-setting",
    title: "unknown setting",
    severity: Error,
  }
}

impl UnknownSettingRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for setting in context.local_declarations(context.settings()) {
      if Builtin::setting(&setting.name.value).is_none() {
        let suggestion = setting
          .name
          .value
          .find_suggestion(Builtin::settings().map(Builtin::name));

        let message = match &suggestion {
          Some(suggestion) => format!(
            "Unknown setting `{}`. Did you mean `{suggestion}`?",
            setting.name.value,
          ),
          None => format!("Unknown setting `{}`", setting.name.value),
        };

        let quickfix = suggestion
          .map(|suggestion| Quickfix::replacement(&setting.name, suggestion));

        diagnostics
          .push(Diagnostic::new(message, setting.range).quickfix(quickfix));
      }
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn settings_boolean_shorthand() {
    AnalyzerTest::new(indoc! {
      "
      set export

      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn settings_default_list_recognized() {
    AnalyzerTest::new(indoc! {
      "
      set default-list

      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn settings_default_script_recognized() {
    AnalyzerTest::new(indoc! {
      "
      set default-script

      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn settings_guards_recognized() {
    AnalyzerTest::new(indoc! {
      "
      set guards

      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn settings_lazy_recognized() {
    AnalyzerTest::new(indoc! {
      "
      set lazy

      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn settings_lists_recognized() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn settings_unknown() {
    AnalyzerTest::new(indoc! {
      "
      set unknown-setting := true

      foo:
        echo \"foo\"
      "
    })
    .error(
      "Unknown setting `unknown-setting`",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn settings_unknown_with_expression() {
    AnalyzerTest::new(indoc! {
      "
      value := 'bar'
      set unknown-setting := value

      foo:
        echo \"foo\"
      "
    })
    .error(
      "Unknown setting `unknown-setting`",
      lsp::Range::at(1, 0, 2, 0),
    )
    .run();
  }
}
