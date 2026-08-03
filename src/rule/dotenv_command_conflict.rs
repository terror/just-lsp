use super::*;

define_rule! {
  DotenvCommandConflictRule {
    id: "dotenv-command-conflict",
    message: "conflicting dotenv command setting",
    run(context) {
      let settings = context
        .settings()
        .iter()
        .map(|setting| {
          (
            setting,
            GroupSet::from_attributes(&setting.attributes),
          )
        })
        .collect::<Vec<_>>();

      settings
        .iter()
        .enumerate()
        .flat_map(|(index, current)| {
          settings[..index]
            .iter()
            .map(move |previous| (previous, current))
        })
        .filter(|((previous, previous_groups), (current, current_groups))| {
          previous_groups.conflicts_with(current_groups)
            && (previous.name.value == "dotenv-command"
              && current.loads_dotenv()
              || current.name.value == "dotenv-command"
                && previous.loads_dotenv())
        })
        .map(|((previous, _), (current, _))| {
          Diagnostic::error(
            format!(
              "`{}` is incompatible with `{}`",
              previous.name.value, current.name.value,
            ),
            current.range,
          )
        })
        .collect()
    }
  }
}
