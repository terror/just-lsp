use super::*;

define_rule! {
  /// Rejects backtick expressions whose evaluated contents begin with a
  /// shebang, which `just` reserves for a future syntax extension.
  BacktickShebangRule {
    id: "backtick-shebang",
    title: "backtick shebang",
    severity: Error,
  }
}

impl BacktickShebangRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let document = context.document();

    context
      .tree()
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

        let is_blank =
          |line: &str| line.trim_matches([' ', '\t', '\r']).is_empty();

        let Some(line) = contents.lines().take(2).find(|line| !is_blank(line))
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
        Diagnostic::new(
          "Backticks may not start with `#!`",
          document.get_range(&command),
        )
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn backtick_shebangs_after_content_are_allowed() {
    AnalyzerTest::new(indoc! {
      "
      foo := ` #!single`
      bar := ```

        #!indented
      ```
      baz := ```
        echo foo
        #!later
      ```
      qux := ```\u{a0}#!unicode-prefix```
      quux := ```
        #!unicode-indentation
      \u{a0}
      ```

      recipe:
        echo {{ foo }} {{ bar }} {{ baz }} {{ qux }} {{ quux }}
      "
    })
    .run();
  }

  #[test]
  fn backticks_may_not_start_with_shebangs() {
    AnalyzerTest::new(indoc! {
      "
      foo := `#!single`
      bar := ```
        #!indented
      ```

      recipe:
        echo {{ foo }} {{ bar }}
      "
    })
    .error(
      "Backticks may not start with `#!`",
      lsp::Range::at(0, 7, 0, 17),
    )
    .error(
      "Backticks may not start with `#!`",
      lsp::Range::at(1, 7, 3, 3),
    )
    .run();
  }
}
