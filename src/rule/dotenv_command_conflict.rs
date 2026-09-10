use super::*;

define_rule! {
  DotenvCommandConflictRule {
    id: "dotenv-command-conflict",
    message: "conflicting dotenv command setting",
    run(context) {
      let mut settings = context
        .settings()
        .iter()
        .map(|setting| {
          let groups = GroupSet::from_attributes(&setting.attributes);

          (setting, groups)
        })
        .collect::<Vec<_>>();

      settings.sort_by_key(|(setting, _)| setting.uri == context.document().uri);

      let mut seen = HashSet::new();

      settings
        .iter()
        .enumerate()
        .flat_map(|(index, current)| {
          settings[..index]
            .iter()
            .map(move |previous| (previous, current))
        })
        .filter(|((previous, previous_groups), (current, current_groups))| {
          current.uri == context.document().uri
            && previous_groups.conflicts_with(current_groups)
            && (previous.name.value == "dotenv-command"
              && current.loads_dotenv()
              || current.name.value == "dotenv-command"
                && previous.loads_dotenv())
        })
        .filter_map(|((previous, _), (current, _))| {
          let message = format!(
            "`{}` is incompatible with `{}`",
            previous.name.value, current.name.value,
          );

          let key = (
            message.clone(),
            current.range.start.line,
            current.range.start.character,
            current.range.end.line,
            current.range.end.character,
          );

          seen
            .insert(key)
            .then(|| Diagnostic::error(message, current.range))
        })
        .collect()
    }
  }
}
