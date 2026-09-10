use super::*;

define_rule! {
  /// Warns when recipe lines use indentation that differs from the first recipe
  /// line, matching the behavior of the `just` parser.
  InconsistentIndentationRule {
    id: "inconsistent-recipe-indentation",
    message: "inconsistent indentation",
    run(context) {
      let default_script = context.setting_enabled("default-script");

      context
        .local_declarations(context.recipes())
        .filter(|recipe| !recipe.runs_as_script(default_script))
        .filter_map(|recipe| {
          let mut lines = recipe
            .body_lines()
            .filter(|line| line.kind != IndentKind::Mixed);

          let expected = lines.next()?;

          lines
            .scan(expected.continues, |continues, line| {
              Some((mem::replace(continues, line.continues), line))
            })
            .find(|(previous_continues, line)| {
              !*previous_continues
                && line.kind == expected.kind
                && line.indent != expected.indent
            })
            .map(|(_, line)| {
              Diagnostic::error(
                format!(
                  "Recipe line has inconsistent leading whitespace. \
                   Recipe started with `{}` but found line with `{}`",
                  Self::visualize_whitespace(expected.indent),
                  Self::visualize_whitespace(line.indent),
                ),
                line.range(),
              )
            })
        })
        .collect()
    }
  }
}

impl InconsistentIndentationRule {
  fn visualize_whitespace(indent: &str) -> String {
    if indent.is_empty() {
      return "∅".to_string();
    }

    indent
      .chars()
      .map(|ch| match ch {
        ' ' => '␠',
        '\t' => '⇥',
        other => other,
      })
      .collect()
  }
}
