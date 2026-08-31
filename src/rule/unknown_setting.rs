use super::*;

define_rule! {
  /// Emits diagnostics for `set` directives targeting settings that don't exist
  /// in the builtin catalog.
  UnknownSettingRule {
    id: "unknown-setting",
    message: "unknown setting",
    provides_quickfixes: true,
    run(context) {
      let mut diagnostics = Vec::new();

      for setting in context.document().settings() {
        if context.builtin_setting(&setting.name.value).is_none() {
          let suggestion = setting.name.value.find_suggestion(
            BUILTINS.iter().filter_map(|builtin| match builtin {
              Builtin::Setting { name, .. } => Some(*name),
              _ => None,
            }),
          );

          let mut diagnostic = Diagnostic::error(
            match &suggestion {
              Some(suggestion) => format!(
                "Unknown setting `{}`. Did you mean `{suggestion}`?",
                setting.name.value,
              ),
              None => format!("Unknown setting `{}`", setting.name.value),
            },
            setting.range,
          );

          if let Some(suggestion) = suggestion {
            diagnostic = diagnostic
              .quickfix(Quickfix::replacement(&setting.name, suggestion));
          }

          diagnostics.push(diagnostic);
        }
      }

      diagnostics
    }
  }
}
