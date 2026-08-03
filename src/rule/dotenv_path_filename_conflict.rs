use super::*;

define_rule! {
  DotenvPathFilenameConflictRule {
    id: "dotenv-path-filename-conflict",
    message: "conflicting dotenv settings",
    run(context) {
      let mut diagnostics = Vec::new();

      let settings = context.settings();

      for (index, setting) in settings.iter().enumerate() {
        let groups = GroupSet::from_attributes(&setting.attributes);

        for previous in &settings[..index] {
          if !GroupSet::from_attributes(&previous.attributes)
            .conflicts_with(&groups)
          {
            continue;
          }

          let filename_range = if previous.name.value == "dotenv-filename"
            && setting.name.value == "dotenv-path"
          {
            Some(previous.range)
          } else if previous.name.value == "dotenv-path"
            && setting.name.value == "dotenv-filename"
          {
            Some(setting.range)
          } else {
            None
          };

          if let Some(range) = filename_range {
            diagnostics.push(Diagnostic::warning(
              "`dotenv-path` overrides `dotenv-filename`".to_string(),
              range,
            ));
          }
        }
      }

      diagnostics
    }
  }
}
