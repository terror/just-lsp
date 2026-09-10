use super::*;

define_rule! {
  /// Surfaces tree-sitter syntax errors and missing nodes so users get feedback
  /// on malformed `justfile` syntax before other rules run.
  SyntaxErrorRule {
    id: "syntax-error",
    message: "syntax errors",
    run(context) {
      let mut diagnostics = Vec::new();

      let mut cursor = context.tree().root_node().walk();

      SyntaxErrorRule::collect(context.document(), &mut cursor, &mut diagnostics);

      diagnostics
    }
  }
}

impl SyntaxErrorRule {
  fn collect(
    document: &Document,
    cursor: &mut TreeCursor<'_>,
    diagnostics: &mut Vec<Diagnostic>,
  ) {
    let node = cursor.node();

    if node.is_error() {
      diagnostics.push(Diagnostic::error(
        SyntaxErrorRule::error_message(document, &node),
        node.get_range(document),
      ));
    }

    if node.is_missing() {
      diagnostics.push(Diagnostic::error(
        SyntaxErrorRule::missing_message(&node),
        node.get_range(document),
      ));
    }

    if cursor.goto_first_child() {
      loop {
        SyntaxErrorRule::collect(document, cursor, diagnostics);

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
    let preview =
      SyntaxErrorRule::snippet_preview(&document.get_node_text(node));

    if let Some(snippet) = preview {
      format!("Syntax error near `{snippet}`")
    } else if let Some(parent) = node.parent() {
      format!(
        "Syntax error in {}",
        SyntaxErrorRule::describe_kind(parent.kind())
      )
    } else {
      "Syntax error".to_string()
    }
  }

  fn missing_message(node: &Node<'_>) -> String {
    let missing = SyntaxErrorRule::describe_kind(node.kind());

    if let Some(parent) = node.parent() {
      let context = SyntaxErrorRule::describe_kind(parent.kind());

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
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");

    if collapsed.is_empty() {
      return None;
    }

    Some(SyntaxErrorRule::truncate(&collapsed, 40))
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
      SyntaxErrorRule::describe_kind("recipe_body_line"),
      "recipe body line"
    );
  }

  #[test]
  fn describe_kind_handles_newline_kind() {
    assert_eq!(SyntaxErrorRule::describe_kind("\n"), "newline");
  }

  #[test]
  fn snippet_preview_collapses_whitespace() {
    assert_eq!(
      SyntaxErrorRule::snippet_preview("  foo\t\tbar \n baz  "),
      Some("foo bar baz".to_string())
    );
  }

  #[test]
  fn snippet_preview_returns_none_for_blank() {
    assert_eq!(SyntaxErrorRule::snippet_preview("   \n\t  "), None);
  }

  #[test]
  fn truncate_limits_length() {
    assert_eq!(
      SyntaxErrorRule::truncate("abcdefghijklmnopqrstuvwxyz", 5),
      "abcde..."
    );
  }
}
