use super::*;

define_rule! {
  /// Emits diagnostics when the same `set` option is declared more than once.
  DuplicateSettingRule {
    id: "duplicate-setting",
    message: "duplicate setting",
    run(context) {
      let mut diagnostics = Vec::new();

      for setting in context.duplicate_declarations(
        context.settings(),
        |setting| &setting.name,
        |setting| &setting.attributes,
      ) {
        diagnostics.push(Diagnostic::error(
          format!("Duplicate setting `{}`", setting.name.value),
          setting.range,
        ));
      }

      diagnostics
    }
  }
}
