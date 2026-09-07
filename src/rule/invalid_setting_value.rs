use super::*;

struct SettingValidation {
  expected: &'static str,
  name: &'static str,
  validate: fn(&str) -> bool,
}

const SETTING_VALIDATIONS: &[SettingValidation] = &[
  SettingValidation {
    expected: "a non-empty whitespace string literal",
    name: "indentation",
    validate: |value| {
      !value.is_empty() && value.chars().all(char::is_whitespace)
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

      for setting in context.settings() {
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
