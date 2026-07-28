use super::*;

define_rule! {
  /// Emits diagnostics when the same `set` option is declared more than once.
  DuplicateSettingRule {
    id: "duplicate-setting",
    message: "duplicate setting",
    run(context) {
      let mut diagnostics = Vec::new();

      let mut groups = HashMap::<String, GroupSet>::new();

      for (setting, current) in context.settings_with_groups() {
        let previous = groups
          .entry(setting.name.value.clone())
          .or_default();

        let duplicate = previous.conflicts_with(current);

        previous.union_with(current.clone());

        if duplicate {
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
