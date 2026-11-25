use super::*;

define_rule! {
  /// Ensures each `set` statement uses the correct value type (boolean, string,
  /// or array) for the targeted builtin setting.
  InvalidSettingTypeRule {
    id: "invalid-setting-type",
    title: "invalid setting kind",
    severity: Error,
  }
}

impl InvalidSettingTypeRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for setting in context.local_declarations(context.settings()) {
      let Some(Builtin::Setting { kind, .. }) =
        Builtin::setting(&setting.name.value)
      else {
        continue;
      };

      if kind.accepts(&setting.kind) {
        continue;
      }

      let article = if *kind == SettingKind::Array {
        "an"
      } else {
        "a"
      };

      diagnostics.push(Diagnostic::new(
        format!(
          "Setting `{}` expects {article} {kind} value",
          setting.name.value
        ),
        setting.range,
      ));
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
  fn analyzer_ignores_imported_invalid_setting_kinds() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "set export := 'bar'\n")
      .run();
  }

  #[test]
  fn settings_array_type_error() {
    AnalyzerTest::new(indoc! {
      "
      set windows-shell := \"bash\"

      foo:
        echo \"foo\"
      "
    })
    .error(
      "Setting `windows-shell` expects an array value",
      lsp::Range::at(0, 0, 1, 0),
    )
    .warning(
      "`windows-shell` is deprecated, use `[windows]` attribute on `set shell` instead",
      lsp::Range::at(0, 4, 0, 17),
    )
    .run();
  }

  #[test]
  fn settings_array_type_error_with_nested_list() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists
      set shell := show(['foo'])

      foo:
        echo \"foo\"
      "
    })
    .error(
      "Setting `shell` expects an array value",
      lsp::Range::at(2, 0, 3, 0),
    )
    .run();
  }

  #[test]
  fn settings_boolean_type_correct() {
    AnalyzerTest::new(indoc! {
      "
      set export := true
      set dotenv-load := false

      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn settings_boolean_type_error() {
    AnalyzerTest::new(indoc! {
      "
      set export := 'foo'

      foo:
        echo \"foo\"
      "
    })
    .error(
      "Setting `export` expects a boolean value",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn settings_boolean_type_error_with_expression() {
    AnalyzerTest::new(indoc! {
      "
      env := 'true'
      set export := env

      foo:
        echo \"foo\"
      "
    })
    .error(
      "Setting `export` expects a boolean value",
      lsp::Range::at(1, 0, 2, 0),
    )
    .run();
  }

  #[test]
  fn settings_dotenv_command_string_type_correct() {
    AnalyzerTest::new(indoc! {
      "
      set dotenv-command := \"sops -d .enc.env\"

      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn settings_dotenv_lists_type_correct() {
    #[track_caller]
    fn case(setting: &str) {
      AnalyzerTest::new(&formatdoc! {
        r#"
        set unstable
        set lists
        set {setting} := ["foo", "bar"]

        foo:
          echo "foo"
        "#
      })
      .run();
    }

    case("dotenv-filename");
    case("dotenv-path");
    case("dotenv-command");
  }

  #[test]
  fn settings_shell_array_accepts_shell_expanded_strings() {
    AnalyzerTest::new(indoc! {
      r#"
      set shell := [x"${SHELL_BIN:-bash}", x"-c"]

      foo:
        echo "foo"
      "#
    })
    .run();
  }

  #[test]
  fn settings_string_type_correct() {
    AnalyzerTest::new(indoc! {
      "
      set dotenv-path := \".env.development\"

      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn settings_string_type_correct_with_expression() {
    AnalyzerTest::new(indoc! {
      "
      env := 'development'
      set dotenv-path := '.env.' + env

      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn settings_string_type_correct_with_shell_expanded_string() {
    AnalyzerTest::new(indoc! {
      r#"
      set dotenv-path := x"~/.env.${JUST_ENV:-development}"

      foo:
        echo "foo"
      "#
    })
    .run();
  }

  #[test]
  fn settings_string_type_error() {
    AnalyzerTest::new(indoc! {
      "
      set dotenv-path := true

      foo:
        echo \"foo\"
      "
    })
    .error(
      "Setting `dotenv-path` expects a string or array value",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }
}
