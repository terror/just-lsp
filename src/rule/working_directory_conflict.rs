use super::*;

define_rule! {
  /// Detects conflicts between working-directory and no-cd directives.
  WorkingDirectoryConflictRule {
    id: "working-directory-conflict",
    message: "conflicting working directory configuration",
    run(context) {
      let mut diagnostics = Vec::new();

      let mut settings = Vec::<&Setting>::new();

      for setting in context.settings() {
        let relevant = setting.name.value == "working-directory"
          || setting.name.value == "no-cd"
            && matches!(setting.kind, SettingKind::Boolean(true));

        if !relevant {
          continue;
        }

        let groups = GroupSet::from_attributes(&setting.attributes);

        if let Some(previous) = settings.iter().find(|previous| {
          previous.name.value != setting.name.value
            && GroupSet::from_attributes(&previous.attributes)
              .conflicts_with(&groups)
        }) {
          diagnostics.push(Diagnostic::error(
            format!(
              "`{}` is incompatible with `{}`",
              previous.name.value, setting.name.value
            ),
            setting.range,
          ));
        }

        settings.push(setting);
      }

      for recipe in context.recipes() {
        let working_directory_attribute =
          recipe.find_attribute("working-directory");

        let no_cd_attribute = recipe.find_attribute("no-cd");

        if let (Some(attribute), Some(_)) =
          (working_directory_attribute, no_cd_attribute)
        {
          diagnostics.push(Diagnostic::error(
            format!(
              "Recipe `{}` can't combine `[working-directory]` with `[no-cd]`",
              recipe.name.value
            ),
            attribute.range,
          ));
        }
      }

      diagnostics
    }
  }
}
