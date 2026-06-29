use super::*;

define_rule! {
  UnstableFeatureGateRule {
    id: "unstable-feature-gate",
    message: "unstable feature used without set unstable",
    run(context) {
      let mut diagnostics = Vec::new();

      let settings = context.document().settings();

      if settings.iter().any(|setting| {
        setting.name.value == "unstable"
          && matches!(setting.kind, SettingKind::Boolean(true))
      }) {
        return diagnostics;
      }

      diagnostics.extend(settings.iter().filter_map(|setting| {
        if setting.name.value == "lists"
          && matches!(setting.kind, SettingKind::Boolean(true))
        {
          Some(Diagnostic::warning(
            "`set lists` is unstable without `set unstable`",
            setting.name.range,
          ))
        } else {
          None
        }
      }));

      diagnostics.extend(context.document().functions().iter().map(|function| {
        Diagnostic::warning(
          format!(
            "User-defined function `{}` is unstable without `set unstable`",
            function.name.value
          ),
          function.name.range,
        )
      }));

      diagnostics
    }
  }
}
