use super::*;

define_rule! {
  /// Emits diagnostics when variable assignments reuse the same name without
  /// explicitly opting into overriding via `allow-duplicate-variables`.
  DuplicateVariableRule {
    id: "duplicate-variable",
    message: "duplicate variable",
    run(context) {
      let allow_duplicates = context.setting_enabled("allow-duplicate-variables");

      if allow_duplicates {
        return Vec::new();
      }

      let (mut diagnostics, mut conflicts) = (Vec::new(), ConflictTracker::default());

      for variable in context.variables() {
        if conflicts.record(&variable.name, &variable.attributes) {
          diagnostics.push(Diagnostic::error(
            format!("Duplicate variable `{}`", variable.name.value),
            variable.range,
          ));
        }
      }

      diagnostics
    }
  }
}
