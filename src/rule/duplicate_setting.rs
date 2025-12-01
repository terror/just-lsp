use super::*;

define_rule! {
  /// Emits diagnostics when the same `set` option is declared more than once.
  DuplicateSettingRule {
    id: "duplicate-setting",
    message: "duplicate setting",
    run(ctx) {
      let mut diagnostics = Vec::new();

      let mut seen = HashSet::new();

      for setting in ctx.settings() {
        if !seen.insert(setting.name.clone()) {
          diagnostics.push(Diagnostic::error(
            format!("Duplicate setting `{}`", setting.name),
            setting.range,
          ));
        }
      }

      diagnostics
    }
  }
}
