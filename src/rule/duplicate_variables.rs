use super::*;

define_rule! {
  /// Emits diagnostics when variable assignments reuse the same name without
  /// explicitly opting into overriding via `allow-duplicate-variables`.
  DuplicateVariableRule {
    id: "duplicate-variable",
    message: "duplicate variable",
    run(context) {
      let (mut diagnostics, mut groups) = (Vec::new(), HashMap::<String, GroupSet>::new());

      for (variable, current) in context.variables_with_groups() {
        let previous = groups
          .entry(variable.name.value.clone())
          .or_default();

        let overlap = previous.intersection(current);

        previous.union_with(current.clone());

        if !overlap.is_empty()
          && !context.setting_enabled_for("allow-duplicate-variables", &overlap)
        {
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
