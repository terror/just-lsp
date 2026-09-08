use super::*;

define_rule! {
  /// Rejects backtick expressions whose evaluated contents begin with a
  /// shebang, which `just` reserves for a future syntax extension.
  BacktickShebangRule {
    id: "backtick-shebang",
    message: "backtick shebang",
    run(context) {
      let Some(tree) = context.tree() else {
        return Vec::new();
      };

      let document = context.document();

      tree
        .root_node()
        .find_all("external_command")
        .into_iter()
        .filter(|command| {
          let command = document.get_node_text(command);

          let Some(contents) = command
            .strip_prefix("```")
            .and_then(|s| s.strip_suffix("```"))
          else {
            return command
              .strip_prefix('`')
              .and_then(|s| s.strip_suffix('`'))
              .is_some_and(|s| s.starts_with("#!"));
          };

          let is_blank = |line: &str| {
            line.trim_matches([' ', '\t', '\r']).is_empty()
          };

          let Some(line) = contents
            .lines()
            .take(2)
            .find(|line| !is_blank(line))
          else {
            return false;
          };

          let text = line.trim_start_matches([' ', '\t']);

          let indent = &line[..line.len() - text.len()];

          text.starts_with("#!")
            && contents
              .lines()
              .all(|line| is_blank(line) || line.starts_with(indent))
        })
        .map(|command| {
          Diagnostic::error(
            "Backticks may not start with `#!`",
            command.get_range(document),
          )
        })
        .collect()
    }
  }
}
