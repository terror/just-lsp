use super::*;

use std::ops::ControlFlow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IndentKind {
  Spaces,
  Tabs,
}

#[derive(Debug)]
struct RecipeLine {
  indent_length: usize,
  kind: Option<IndentKind>,
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

    if indent.is_empty() {
      return None;
    }

    let kind = match (indent.contains(' '), indent.contains('\t')) {
      (true, true) => None,
      (true, false) => Some(IndentKind::Spaces),
      (false, true) => Some(IndentKind::Tabs),
      (false, false) => return None,
    };

    Some(Self {
      indent_length: indent.len(),
      kind,
      relative_line,
    })
  }
}

define_rule! {
  /// Detects recipes that mix tabs and spaces for indentation, which often
  /// results in confusing or invalid `just` bodies.
  MixedIndentationRule {
    id: "mixed-recipe-indentation",
    message: "mixed indentation",
    run(context) {
      context
        .recipes()
        .iter()
        .filter(|recipe| recipe.shebang.is_none())
        .filter_map(Self::find_mixed_indentation)
        .collect()
    }
  }
}

impl MixedIndentationRule {
  fn find_mixed_indentation(recipe: &Recipe) -> Option<Diagnostic> {
    let body_start_line = recipe.range.start.line + 1;

    Self::recipe_body_lines(&recipe.content)
      .try_fold(None, |expected_kind: Option<IndentKind>, line| {
        let absolute_line = body_start_line + line.relative_line;

        let Some(line_kind) = line.kind else {
          return ControlFlow::Break(Self::make_diagnostic(
            &recipe.name.value,
            absolute_line,
            line.indent_length,
          ));
        };

        match expected_kind {
          None => ControlFlow::Continue(Some(line_kind)),
          Some(expected) if expected != line_kind => {
            ControlFlow::Break(Self::make_diagnostic(
              &recipe.name.value,
              absolute_line,
              line.indent_length,
            ))
          }
          _ => ControlFlow::Continue(expected_kind),
        }
      })
      .break_value()
  }

  fn make_diagnostic(
    recipe_name: &str,
    line: u32,
    indent_length: usize,
  ) -> Diagnostic {
    let indent = u32::try_from(indent_length).unwrap_or(u32::MAX);

    let range = lsp::Range {
      start: lsp::Position { line, character: 0 },
      end: lsp::Position {
        line,
        character: indent,
      },
    };

    Diagnostic::error(
      format!("Recipe `{recipe_name}` mixes tabs and spaces for indentation"),
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
}
