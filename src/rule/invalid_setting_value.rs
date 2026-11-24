use super::*;

struct SettingValidation {
  expected: &'static str,
  name: &'static str,
  validate: fn(&str) -> bool,
}

const SETTING_VALIDATIONS: &[SettingValidation] = &[
  SettingValidation {
    expected: "`all` or `recipes`",
    name: "dry-run",
    validate: |value| matches!(value, "all" | "recipes"),
  },
  SettingValidation {
    expected: "a non-empty string literal containing only spaces or only tabs",
    name: "indentation",
    validate: |value| {
      value.bytes().next().is_some_and(|first| {
        matches!(first, b' ' | b'\t') && value.bytes().all(|byte| byte == first)
      })
    },
  },
  SettingValidation {
    expected: "a valid `MAJOR.MINOR.PATCH` version",
    name: "minimum-version",
    validate: |value| {
      fn valid_component(component: &str) -> bool {
        match component.as_bytes() {
          [b'0'] => true,
          [first, rest @ ..] => {
            component.len() <= 9
              && first.is_ascii_digit()
              && *first != b'0'
              && rest.iter().all(u8::is_ascii_digit)
          }
          [] => false,
        }
      }

      let mut components = value.split('.');

      components.by_ref().take(3).all(valid_component)
        && components.next().is_none()
        && value.bytes().filter(|byte| *byte == b'.').count() == 2
    },
  },
];

define_rule! {
  /// Ensures settings with constrained values use a supported literal value.
  InvalidSettingValueRule {
    id: "invalid-setting-value",
    message: "invalid setting value",
    run(context) {
      let mut diagnostics = Vec::new();

      for setting in context.local_declarations(context.settings()) {
        if Self::empty_interpreter(context, setting) {
          diagnostics.push(Diagnostic::error(
            format!(
              "Setting `{}` must be a non-empty array",
              setting.name.value
            ),
            setting.value.range,
          ));

          continue;
        }

        let Some(validation) = SETTING_VALIDATIONS
          .iter()
          .find(|validation| validation.name == setting.name.value)
        else {
          continue;
        };

        let valid = setting
          .value
          .value
          .literal()
          .is_some_and(|value| (validation.validate)(&value));

        if valid {
          continue;
        }

        diagnostics.push(Diagnostic::error(
          format!(
            "Setting `{}` must be {}",
            validation.name, validation.expected,
          ),
          setting.value.range,
        ));
      }

      diagnostics
    }
  }
}

impl InvalidSettingValueRule {
  fn empty_interpreter(context: &RuleContext<'_>, setting: &Setting) -> bool {
    matches!(
      setting.name.value.as_str(),
      "script-interpreter" | "shell" | "windows-shell"
    ) && setting.kind == SettingKind::Array
      && context
        .document()
        .node_at_position(setting.value.range.start)
        .is_some_and(|node| {
          node
            .get_parent("list_literal")
            .is_some_and(|list| list.child_by_field_name("elements").is_none())
        })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn analyzer_ignores_imported_invalid_setting_values() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "set indentation := 'foo'\n")
      .run();
  }

  #[test]
  fn settings_dry_run_value_validation() {
    #[track_caller]
    fn case(value: &str, valid: bool) {
      let content = format!("set dry-run := {value}\n");

      let test = AnalyzerTest::new(&content);

      let test = if valid {
        test
      } else {
        test.error(
          "Setting `dry-run` must be `all` or `recipes`",
          lsp::Range::at(0, 15, 0, 15 + u32::try_from(value.len()).unwrap()),
        )
      };

      test.run();
    }

    case("'all'", true);
    case("'recipes'", true);
    case("'foo'", false);
    case("'all' + ''", false);
  }

  #[test]
  fn settings_indentation_value_validation() {
    #[track_caller]
    fn case(value: &str, valid: bool) {
      let content = format!("set indentation := {value}\n");

      let test = AnalyzerTest::new(&content);

      let test = if valid {
        test
      } else {
        test.error(
          "Setting `indentation` must be a non-empty string literal containing only spaces or only tabs",
          lsp::Range::at(0, 19, 0, 19 + u32::try_from(value.len()).unwrap()),
        )
      };

      test.run();
    }

    case(r#""  ""#, true);
    case(r#""\t\t""#, true);
    case(r#""\u{20}\u{20}""#, true);
    case(r#"" \t""#, false);
    case(r#""\t ""#, false);
    case(r#""\n""#, false);
    case(r#""\u{2003}""#, false);
    case("'  '", true);
    case(r#""""#, false);
    case(r#""foo""#, false);
    case(r#"" " + " ""#, false);
  }

  #[test]
  fn settings_interpreter_empty() {
    #[track_caller]
    fn case(setting: &str, message: &'static str) {
      let content = format!("set {setting} := []\n");

      let start = 8 + u32::try_from(setting.len()).unwrap();

      AnalyzerTest::new(&content)
        .error(message, lsp::Range::at(0, start, 0, start + 2))
        .run();
    }

    case(
      "script-interpreter",
      "Setting `script-interpreter` must be a non-empty array",
    );
    case("shell", "Setting `shell` must be a non-empty array");

    AnalyzerTest::new("set windows-shell := []\n")
      .warning(
        "`windows-shell` is deprecated, use `[windows]` attribute on `set shell` instead",
        lsp::Range::at(0, 4, 0, 17),
      )
      .error(
        "Setting `windows-shell` must be a non-empty array",
        lsp::Range::at(0, 21, 0, 23),
      )
      .run();
  }

  #[test]
  fn settings_minimum_version_value_validation() {
    #[track_caller]
    fn case(value: &str, valid: bool) {
      let content = format!("set minimum-version := {value}\n");

      let test = AnalyzerTest::new(&content);

      let test = if valid {
        test
      } else {
        test.error(
          "Setting `minimum-version` must be a valid `MAJOR.MINOR.PATCH` version",
          lsp::Range::at(0, 23, 0, 23 + u32::try_from(value.len()).unwrap()),
        )
      };

      test.run();
    }

    case(r#""0.0.0""#, true);
    case(r#""1.55.0""#, true);
    case(r#""999999999.0.0""#, true);
    case("'1.2.3'", true);
    case(r#""""#, false);
    case(r#""01.2.3""#, false);
    case(r#""1.2""#, false);
    case(r#""1.2.3.4""#, false);
    case(r#""1000000000.0.0""#, false);
    case(r#""1.2.3-pre""#, false);
    case(r#""1.2.3+build""#, false);
    case(r#""1" + ".2.3""#, false);
  }
}
