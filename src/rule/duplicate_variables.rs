use super::*;

define_rule! {
  /// Emits diagnostics when variable assignments reuse the same name without
  /// explicitly opting into overriding via `allow-duplicate-variables`.
  DuplicateVariableRule {
    id: "duplicate-variable",
    message: "duplicate variable",
    phase: RulePhase::Project,
    run(context) {
      let allow_duplicates = context.setting_enabled("allow-duplicate-variables");

      if allow_duplicates {
        return Vec::new();
      }

      let (mut diagnostics, mut groups) = (Vec::new(), HashMap::<String, GroupSet>::new());

      for variable in context.variables() {
        let current = GroupSet::from_attributes(&variable.attributes);

        let previous = groups
          .entry(variable.name.value.clone())
          .or_default();

        let duplicate = previous.conflicts_with(&current);

        previous.union_with(current);

        if duplicate {
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
