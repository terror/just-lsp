use super::*;

define_rule! {
  /// Emits diagnostics when the same `set` option is declared more than once.
  DuplicateSettingRule {
    id: "duplicate-setting",
    message: "duplicate setting",
    run(context) {
      let mut diagnostics = Vec::new();

      let mut conflicts = ConflictTracker::default();

      for setting in context.settings() {
        if conflicts.record(&setting.name, &setting.attributes) {
          diagnostics.push(Diagnostic::error(
            format!("Duplicate setting `{}`", setting.name.value),
            setting.range,
          ));
        }
      }

      diagnostics
    }
  }
}
