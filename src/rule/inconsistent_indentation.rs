use super::*;

/// Warns when recipe lines use indentation that differs from the first recipe
/// line, matching the behavior of the `just` parser.
pub(crate) struct InconsistentIndentationRule;

impl Rule for InconsistentIndentationRule {
  fn display_name(&self) -> &'static str {
    "Inconsistent Recipe Indentation"
  }

  fn id(&self) -> &'static str {
    "inconsistent-recipe-indentation"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let Some(tree) = context.tree() else {
      return diagnostics;
    };

    let document = context.document();

    for recipe_node in tree.root_node().find_all("recipe") {
      if let Some(diagnostic) = self.inspect_recipe(document, &recipe_node) {
        diagnostics.push(diagnostic);
      }
    }

    diagnostics
  }
}

impl InconsistentIndentationRule {
  fn diagnostic_for_line(
    &self,
    expected: &str,
    found: &str,
    line: u32,
  ) -> lsp::Diagnostic {
    let indent_chars = u32::try_from(found.chars().count()).unwrap_or(u32::MAX);

    let range = lsp::Range {
      start: lsp::Position { line, character: 0 },
      end: lsp::Position {
        line,
        character: indent_chars,
      },
    };

    self.diagnostic(lsp::Diagnostic {
      range,
      severity: Some(lsp::DiagnosticSeverity::ERROR),
      message: format!(
        "Recipe line has inconsistent leading whitespace. Recipe started with `{}` but found line with `{}`",
        Self::visualize_whitespace(expected),
        Self::visualize_whitespace(found)
      ),
      ..Default::default()
    })
  }

  fn inspect_recipe(
    &self,
    document: &Document,
    recipe_node: &Node<'_>,
  ) -> Option<lsp::Diagnostic> {
    let mut expected_indent: Option<(String, IndentKind)> = None;

    let header_node = recipe_node.find("recipe_header")?;

    let header_end_line = header_node.get_range().end.line;

    let mut line_idx =
      usize::try_from(header_end_line.saturating_add(1)).ok()?;

    while line_idx < document.content.len_lines() {
      let line_text = document.content.line(line_idx).to_string();

      let line = line_text.trim_end_matches(['\r', '\n']);

      if line.trim().is_empty() {
        line_idx += 1;
        continue;
      }

      if !matches!(line.chars().next(), Some(' ' | '\t')) {
        break;
      }

      let indent = line
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect::<String>();

      if indent.is_empty() {
        line_idx += 1;
        continue;
      }

      let Some(kind) = IndentKind::from_indent(&indent) else {
        line_idx += 1;
        continue;
      };

      match &mut expected_indent {
        None => expected_indent = Some((indent, kind)),
        Some((expected, expected_kind)) => {
          if *expected_kind != kind {
            line_idx += 1;
            continue;
          }

          if *expected != indent {
            return Some(self.diagnostic_for_line(
              expected.as_str(),
              &indent,
              u32::try_from(line_idx).unwrap_or(u32::MAX),
            ));
          }
        }
      }

      line_idx += 1;
    }

    None
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum IndentKind {
  Spaces,
  Tabs,
}

impl IndentKind {
  fn from_indent(indent: &str) -> Option<Self> {
    if indent.is_empty() {
      return None;
    }

    let (has_space, has_tab) = (
      indent.chars().any(|ch| ch == ' '),
      indent.chars().any(|ch| ch == '\t'),
    );

    match (has_space, has_tab) {
      (true, false) => Some(Self::Spaces),
      (false, true) => Some(Self::Tabs),
      (true, true) | (false, false) => None,
    }
  }
}
