use super::*;

struct SettingValidation {
  expected: &'static str,
  name: &'static str,
  validate: fn(&str) -> bool,
}

const SETTING_VALIDATIONS: &[SettingValidation] = &[SettingValidation {
  expected: "a non-empty whitespace string literal",
  name: "indentation",
  validate: |value| !value.is_empty() && value.chars().all(char::is_whitespace),
}];

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
