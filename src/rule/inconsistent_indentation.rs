use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IndentKind {
  Spaces,
  Tabs,
}

impl IndentKind {
  fn from_indent(indent: &str) -> Option<Self> {
    let first_char = indent.chars().next()?;

    if !indent.chars().all(|c| c == first_char) {
      return None;
    }

    match first_char {
      ' ' => Some(Self::Spaces),
      '\t' => Some(Self::Tabs),
      _ => None,
    }
  }
}

#[derive(Debug)]
struct RecipeLine {
  continues: bool,
  indent: String,
  kind: IndentKind,
  relative_line: u32,
}

impl RecipeLine {
  fn parse(relative_line: u32, line: &str) -> Option<Self> {
    if line.trim().is_empty() {
      return None;
    }

    let indent: String = line
      .chars()
      .take_while(|c| *c == ' ' || *c == '\t')
      .collect();

    let kind = IndentKind::from_indent(&indent)?;

    Some(Self {
      continues: line.trim_end().ends_with('\\'),
      indent,
      kind,
      relative_line,
    })
  }
}

#[derive(Debug)]
struct ScanState {
  expected_indent: String,
  expected_kind: IndentKind,
  previous_continues: bool,
}

impl ScanState {
  fn check(&self, line: &RecipeLine, absolute_line: u32) -> Option<Diagnostic> {
    if self.expected_kind != line.kind {
      return None;
    }

    if self.expected_indent != line.indent && !self.previous_continues {
      return Some(InconsistentIndentationRule::make_diagnostic(
        &self.expected_indent,
        &line.indent,
        absolute_line,
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
      context
        .recipes()
        .iter()
        .filter(|recipe| recipe.shebang.is_none())
        .filter_map(Self::find_inconsistent_indentation)
        .collect()
    }
  }
}

impl InconsistentIndentationRule {
  fn find_inconsistent_indentation(recipe: &Recipe) -> Option<Diagnostic> {
    let body_start_line = recipe.range.start.line + 1;

    Self::recipe_body_lines(&recipe.content)
      .try_fold(None, |state: Option<ScanState>, line| {
        let absolute_line = body_start_line + line.relative_line;

        match state {
          None => ControlFlow::Continue(Some(ScanState {
            expected_indent: line.indent,
            expected_kind: line.kind,
            previous_continues: line.continues,
          })),
          Some(state) => {
            if let Some(diagnostic) = state.check(&line, absolute_line) {
              return ControlFlow::Break(diagnostic);
            }

            ControlFlow::Continue(Some(ScanState {
              previous_continues: line.continues,
              ..state
            }))
          }
        }
      })
      .break_value()
  }

  fn make_diagnostic(expected: &str, found: &str, line: u32) -> Diagnostic {
    let indent_chars = u32::try_from(found.chars().count()).unwrap_or(u32::MAX);

    let range = lsp::Range {
      start: lsp::Position { line, character: 0 },
      end: lsp::Position {
        line,
        character: indent_chars,
      },
    };

    Diagnostic::error(
      format!(
        "Recipe line has inconsistent leading whitespace. \
       Recipe started with `{}` but found line with `{}`",
        Self::visualize_whitespace(expected),
        Self::visualize_whitespace(found)
      ),
      range,
    )
  }

  fn recipe_body_lines(content: &str) -> impl Iterator<Item = RecipeLine> + '_ {
    content
    .lines()
    .enumerate()
    .skip(1) // Skip header line
    .take_while(|(_, line)| {
      line.is_empty() || matches!(line.chars().next(), Some(' ' | '\t'))
    })
    .filter_map(|(idx, line)| {
      RecipeLine::parse(u32::try_from(idx).unwrap_or(u32::MAX), line)
    })
  }

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
