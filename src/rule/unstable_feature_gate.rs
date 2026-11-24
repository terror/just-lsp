use super::*;

define_rule! {
  UnstableFeatureGateRule {
    id: "unstable-feature-gate",
    message: "unstable feature used without set unstable",
    run(context) {
      let mut diagnostics = Vec::new();

      if context.setting_enabled("unstable") {
        return diagnostics;
      }

      for setting in context.local_declarations(context.settings()) {
        if setting.name.value == "lists"
          && matches!(setting.kind, SettingKind::Boolean(true))
        {
          diagnostics.push(Diagnostic::warning(
            "`set lists` is unstable without `set unstable`",
            setting.name.range,
          ));
        }
      }

      for function in context.local_declarations(context.functions()) {
        diagnostics.push(Diagnostic::warning(
          format!(
            "User-defined function `{}` is unstable without `set unstable`",
            function.name.value
          ),
          function.name.range,
        ));
      }

      for attribute in context.attributes() {
        if attribute.name.value == "cache"
          && attribute.target == Some(AttributeTarget::Recipe)
        {
          diagnostics.push(Diagnostic::warning(
            "`[cache]` is unstable without `set unstable`",
            attribute.name.range,
          ));
        }
      }

      diagnostics
    }
  }
}
