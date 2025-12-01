use super::*;

/// Detects recipes that mix tabs and spaces for indentation, which often
/// results in confusing or invalid `just` bodies.
pub(crate) struct MixedIndentationRule;

impl Rule for MixedIndentationRule {
  fn id(&self) -> &'static str {
    "mixed-recipe-indentation"
  }

  fn message(&self) -> &'static str {
    "mixed indentation"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let Some(tree) = context.tree() else {
      return diagnostics;
    };

    let document = context.document();

    for recipe_node in tree.root_node().find_all("recipe") {
      if recipe_node.find("recipe_body").is_none() {
        continue;
      }

      if let Some(diagnostic) = self.inspect_recipe(document, &recipe_node) {
        diagnostics.push(diagnostic);
      }
    }

    diagnostics
  }
}

impl MixedIndentationRule {
  fn diagnostic_for_line(
    &self,
    recipe_name: &str,
    line: u32,
    indent_length: usize,
  ) -> lsp::Diagnostic {
    let indent = u32::try_from(indent_length).unwrap_or(u32::MAX);

    let range = lsp::Range {
      start: lsp::Position { line, character: 0 },
      end: lsp::Position {
        line,
        character: indent,
      },
    };

    self.diagnostic(lsp::Diagnostic {
      range,
      severity: Some(lsp::DiagnosticSeverity::ERROR),
      message: format!(
        "Recipe `{recipe_name}` mixes tabs and spaces for indentation"
      ),
      ..Default::default()
    })
  }

  fn inspect_recipe(
    &self,
    document: &Document,
    recipe_node: &Node<'_>,
  ) -> Option<lsp::Diagnostic> {
    let recipe_name =
      recipe_node.find("recipe_header > identifier").map_or_else(
        || "recipe".to_string(),
        |node| document.get_node_text(&node),
      );

    let mut indent_style: Option<IndentStyle> = None;

    for line_node in recipe_node.find_all("recipe_line") {
      let line_range = line_node.get_range(document);

      let Ok(line_idx) = usize::try_from(line_range.start.line) else {
        continue;
      };

      if line_idx >= document.content.len_lines() {
        continue;
      }

      let line = document.content.line(line_idx).to_string();

      if line.trim().is_empty() {
        continue;
      }

      let mut indent_length = 0usize;

      let (mut has_space, mut has_tab) = (false, false);

      for ch in line.chars() {
        match ch {
          ' ' => {
            indent_length += 1;
            has_space = true;
          }
          '\t' => {
            indent_length += 1;
            has_tab = true;
          }
          _ => break,
        }
      }

      if indent_length == 0 {
        continue;
      }

      if has_space && has_tab {
        return Some(self.diagnostic_for_line(
          &recipe_name,
          line_range.start.line,
          indent_length,
        ));
      }

      let current_style = if has_space {
        IndentStyle::Spaces
      } else if has_tab {
        IndentStyle::Tabs
      } else {
        continue;
      };

      match indent_style {
        None => indent_style = Some(current_style),
        Some(style) if style != current_style => {
          return Some(self.diagnostic_for_line(
            &recipe_name,
            line_range.start.line,
            indent_length,
          ));
        }
        _ => {}
      }
    }

    None
  }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IndentStyle {
  Spaces,
  Tabs,
}
