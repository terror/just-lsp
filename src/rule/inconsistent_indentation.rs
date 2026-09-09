use super::*;

#[derive(Debug)]
struct ScanState<'a> {
  expected: RecipeLine<'a>,
  previous_continues: bool,
}

impl ScanState<'_> {
  fn check(&self, line: &RecipeLine) -> Option<Diagnostic> {
    if self.expected.kind != line.kind {
      return None;
    }

    if self.expected.indent != line.indent && !self.previous_continues {
      return Some(Diagnostic::error(
        format!(
          "Recipe line has inconsistent leading whitespace. \
           Recipe started with `{}` but found line with `{}`",
          InconsistentIndentationRule::visualize_whitespace(
            self.expected.indent
          ),
          InconsistentIndentationRule::visualize_whitespace(line.indent),
        ),
        line.range(),
      ));
    }

    None
  }
}

define_rule! {
  /// Warns when recipe lines use indentation that differs from the first recipe
  /// line, matching the behavior of the `just` parser.
  InconsistentIndentationRule {
    id: "inconsistent-recipe-indentation",
    message: "inconsistent indentation",
    run(context) {
      let default_script = context.setting_enabled("default-script");

      context
        .recipes()
        .iter()
        .filter(|recipe| !recipe.runs_as_script(default_script))
        .filter_map(|recipe| {
          recipe
            .body_lines()
            .filter(|line| line.kind != IndentKind::Mixed)
            .try_fold(None, |state: Option<ScanState>, line| match state {
              None => ControlFlow::Continue(Some(ScanState {
                previous_continues: line.continues,
                expected: line,
              })),
              Some(state) => {
                if let Some(diagnostic) = state.check(&line) {
                  return ControlFlow::Break(diagnostic);
                }

                ControlFlow::Continue(Some(ScanState {
                  previous_continues: line.continues,
                  ..state
                }))
              }
            })
            .break_value()
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
