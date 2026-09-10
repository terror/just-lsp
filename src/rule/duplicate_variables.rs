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

      let mut diagnostics = Vec::new();

      for variable in context.duplicate_declarations(
        context.variables(),
        |variable| &variable.name,
        |variable| &variable.attributes,
      ) {
        diagnostics.push(Diagnostic::error(
          format!("Duplicate variable `{}`", variable.name.value),
          variable.range,
        ));
      }

      diagnostics
    }
  }
}
