use super::*;

define_rule! {
  DotenvCommandConflictRule {
    id: "dotenv-command-conflict",
    message: "conflicting dotenv command setting",
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

          if (previous.name.value == "dotenv-command"
            && setting.loads_dotenv())
            || (setting.name.value == "dotenv-command"
              && previous.loads_dotenv())
          {
            diagnostics.push(Diagnostic::error(
              format!(
                "`{}` is incompatible with `{}`",
                previous.name.value, setting.name.value
              ),
              setting.range,
            ));
          }
        }
      }

      diagnostics
    }
  }
}
