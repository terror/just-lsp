use super::*;

define_rule! {
  /// Detects conflicts between working-directory and no-cd directives.
  WorkingDirectoryConflictRule {
    id: "working-directory-conflict",
    message: "conflicting working directory configuration",
    run(context) {
      let settings = context
        .settings()
        .iter()
        .map(|setting| {
          let groups = GroupSet::from_attributes(&setting.attributes);

          (setting, groups)
        })
        .collect::<Vec<_>>();

      let mut seen = HashSet::new();

      let incompatible = |left: &Setting, right: &Setting| {
        match (left.name.value.as_str(), right.name.value.as_str()) {
          ("working-directory", "no-cd") => {
            matches!(right.kind, SettingKind::Boolean(true))
          }
          ("no-cd", "working-directory") => {
            matches!(left.kind, SettingKind::Boolean(true))
          }
          _ => false,
        }
      };

      let mut diagnostics = settings
        .iter()
        .enumerate()
        .flat_map(|(index, current)| {
          settings[..index]
            .iter()
            .map(move |previous| (index, previous, current))
        })
        .filter(|(_, (previous, previous_groups), (current, current_groups))| {
          previous_groups.conflicts_with(current_groups)
            && incompatible(previous, current)
        })
        .filter_map(|(index, (previous, _), (current, _))| {
          let message = format!(
            "`{}` is incompatible with `{}`",
            previous.name.value, current.name.value,
          );

          seen
            .insert((message.clone(), index))
            .then(|| Diagnostic::error(message, current.range))
        })
        .collect::<Vec<_>>();

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
