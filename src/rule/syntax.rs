use super::*;

/// Surfaces tree-sitter syntax errors and missing nodes so users get feedback
/// on malformed `justfile` syntax before other rules run.
pub(crate) struct SyntaxRule;

impl Rule for SyntaxRule {
  fn id(&self) -> &'static str {
    "syntax-errors"
  }

  fn message(&self) -> &'static str {
    "syntax errors"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    if let Some(tree) = context.tree() {
      let document = context.document();
      let mut cursor = tree.root_node().walk();
      Self::collect(document, &mut cursor, &mut diagnostics);
    }

    diagnostics
  }
}

impl SyntaxRule {
  fn collect(
    document: &Document,
    cursor: &mut TreeCursor<'_>,
    diagnostics: &mut Vec<Diagnostic>,
  ) {
    let node = cursor.node();

    if node.is_error() {
      diagnostics.push(Diagnostic::error(
        Self::error_message(document, &node),
        node.get_range(document),
      ));
    }

    if node.is_missing() {
      diagnostics.push(Diagnostic::error(
        Self::missing_message(&node),
        node.get_range(document),
      ));
    }

    if cursor.goto_first_child() {
      loop {
        Self::collect(document, cursor, diagnostics);

        if !cursor.goto_next_sibling() {
          break;
        }
      }

      cursor.goto_parent();
    }
  }

  fn describe_kind(kind: &str) -> String {
    if kind == "\n" {
      return "newline".to_string();
    }

    if kind.chars().all(|char| {
      char.is_ascii_alphanumeric() || char == '_' || char == '-' || char == ' '
    }) {
      let mut name = kind.replace('_', " ");

      if name.is_empty() {
        name = "syntax element".to_string();
      }

      return name;
    }

    format!("`{kind}`")
  }

  fn error_message(document: &Document, node: &Node<'_>) -> String {
    let preview = Self::snippet_preview(&document.get_node_text(node));

    if let Some(snippet) = preview {
      format!("Syntax error near `{snippet}`")
    } else if let Some(parent) = node.parent() {
      format!("Syntax error in {}", Self::describe_kind(parent.kind()))
    } else {
      "Syntax error".to_string()
    }
  }

  fn missing_message(node: &Node<'_>) -> String {
    let missing = Self::describe_kind(node.kind());

    if let Some(parent) = node.parent() {
      let context = Self::describe_kind(parent.kind());

      if missing == context {
        format!("Missing {missing}")
      } else {
        format!("Missing {missing} in {context}")
      }
    } else {
      format!("Missing {missing}")
    }
  }

  fn snippet_preview(text: &str) -> Option<String> {
    let trimmed = text.trim();

    if trimmed.is_empty() {
      return None;
    }

    let mut collapsed = String::new();
    let mut previous_space = false;

    for char in trimmed.chars() {
      if char.is_whitespace() {
        if !previous_space {
          collapsed.push(' ');
          previous_space = true;
        }
      } else {
        collapsed.push(char);
        previous_space = false;
      }
    }

    let collapsed = collapsed.trim();

    if collapsed.is_empty() {
      return None;
    }

    Some(Self::truncate(collapsed, 40))
  }

  fn truncate(text: &str, max_chars: usize) -> String {
    let mut truncated = String::new();

    for (char_count, ch) in text.chars().enumerate() {
      if char_count >= max_chars {
        truncated.push_str("...");
        return truncated;
      }

      truncated.push(ch);
    }

    truncated
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn describe_kind_formats_identifier_like_kinds() {
    assert_eq!(
      SyntaxRule::describe_kind("recipe_body_line"),
      "recipe body line"
    );
  }

  #[test]
  fn describe_kind_handles_newline_kind() {
    assert_eq!(SyntaxRule::describe_kind("\n"), "newline");
  }

  #[test]
  fn snippet_preview_collapses_whitespace() {
    assert_eq!(
      SyntaxRule::snippet_preview("  foo\t\tbar \n baz  "),
      Some("foo bar baz".to_string())
    );
  }

  #[test]
  fn snippet_preview_returns_none_for_blank() {
    assert_eq!(SyntaxRule::snippet_preview("   \n\t  "), None);
  }

  #[test]
  fn truncate_limits_length() {
    assert_eq!(
      SyntaxRule::truncate("abcdefghijklmnopqrstuvwxyz", 5),
      "abcde..."
    );
  }
}
