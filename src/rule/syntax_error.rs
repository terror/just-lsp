use super::*;

define_rule! {
  /// Surfaces tree-sitter syntax errors and missing nodes so users get feedback
  /// on malformed `justfile` syntax before other rules run.
  SyntaxErrorRule {
    id: "syntax-error",
    title: "syntax errors",
    severity: Error,
  }
}

impl SyntaxErrorRule {
  fn chained_comparison<'a>(node: &Node<'a>) -> Option<Node<'a>> {
    let operator = SyntaxErrorRule::comparison_operator(node)?;

    let lhs = node.child(0).filter(|lhs| lhs.kind() == "expression")?;

    SyntaxErrorRule::comparison_operator(&lhs)?;

    Some(operator)
  }

  fn comparison_operator<'a>(node: &Node<'a>) -> Option<Node<'a>> {
    node
      .child(1)
      .filter(|child| ["==", "!=", "=~", "!~"].contains(&child.kind()))
  }

  fn deepest_error(node: Node<'_>) -> Node<'_> {
    let mut cursor = node.walk();

    match node.children(&mut cursor).find(Node::has_error) {
      Some(child) => SyntaxErrorRule::deepest_error(child),
      None => node,
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

  fn operator_after_if<'a>(node: &Node<'a>) -> Option<Node<'a>> {
    let mut current = *node;

    loop {
      let parent = current
        .parent()
        .filter(|parent| parent.kind() == "expression")?;

      if let Some(sibling) = current.next_sibling() {
        return ["+", "++", "/"]
          .contains(&sibling.kind())
          .then_some(sibling);
      }

      current = parent;
    }
  }

  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let document = context.document();

    let root = context.tree().root_node();

    let mut diagnostics = Vec::new();

    let _ = TreeWalker::new(root).walk(|node, _| {
      if node.is_error() {
        diagnostics.push(Diagnostic::new(
          SyntaxErrorRule::error_message(document, &node),
          document.get_range(&node),
        ));
      }

      if node.kind() == "numeric_error"
        && let Some(start) = document.get_node_text(&node).chars().next()
      {
        let range = document.get_range(&node);

        diagnostics.push(Diagnostic::new(
          format!("Unknown start of token `{start}`"),
          lsp::Range {
            end: lsp::Position {
              character: range.start.character + 1,
              ..range.start
            },
            ..range
          },
        ));
      }

      if node.kind() == "dependencies" {
        for marker in TreeWalker::new(node).find_all("^&&").into_iter().skip(1) {
          diagnostics.push(Diagnostic::new(
            "Expected comment, end of file, end of line, or dependency, but found `&&`",
            document.get_range(&marker),
          ));
        }
      }

      if node.is_missing() {
        diagnostics.push(Diagnostic::new(
          SyntaxErrorRule::missing_message(&node),
          document.get_range(&node),
        ));
      }

      if node.kind() == "expression"
        && let Some(operator) = SyntaxErrorRule::chained_comparison(&node)
      {
        diagnostics.push(Diagnostic::new(
          format!(
            "Expected `&&`, `||`, `+`, `++`, or `/`, but found `{}`",
            operator.kind()
          ),
          document.get_range(&operator),
        ));
      }

      if node.kind() == "if_expression"
        && let Some(operator) = SyntaxErrorRule::operator_after_if(&node)
      {
        diagnostics.push(Diagnostic::new(
          format!(
            "Expected `&&`, `!=`, `!~`, `||`, `==`, or `=~`, but found `{}`",
            operator.kind()
          ),
          document.get_range(&operator),
        ));
      }

      ControlFlow::<(), bool>::Continue(true)
    });

    if diagnostics.is_empty() && root.has_error() {
      let node = SyntaxErrorRule::deepest_error(root);

      let end = document.get_range(&node).end;

      diagnostics.push(Diagnostic::new(
        format!(
          "Missing newline in {}",
          SyntaxErrorRule::describe_kind(node.kind())
        ),
        lsp::Range { start: end, end },
      ));
    }

    diagnostics
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
  use {
    super::*,
    indoc::{formatdoc, indoc},
    pretty_assertions::assert_eq,
  };

  #[test]
  fn attribute_arguments_malformed() {
    #[track_caller]
    fn case(attribute: &str, message: &'static str, range: lsp::Range) {
      AnalyzerTest::new(&formatdoc! {
        "
        {attribute}
        foo bar:
          echo {{{{bar}}}}
        "
      })
      .error(message, range)
      .run();
    }

    case(
      "[doc(,)]",
      "Missing identifier in value",
      lsp::Range::at(0, 5, 0, 5),
    );
    case(
      "[arg('bar', short,,)]",
      "Missing identifier in attribute named param",
      lsp::Range::at(0, 18, 0, 18),
    );
    case(
      "[private(),]",
      "Syntax error near `,`",
      lsp::Range::at(0, 10, 0, 11),
    );
  }

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
  fn escaped_braces_are_treated_as_literal_text() {
    AnalyzerTest::new(indoc! {
      "
      test:
        echo \"{{{{hello}}\"
      "
    })
    .run();
  }

  #[test]
  fn missing_newline_between_items() {
    AnalyzerTest::new(indoc! {
      "
      foo := 'bar' baz := 'bob'
      "
    })
    .warning("Variable `foo` appears unused", lsp::Range::at(0, 0, 0, 3))
    .error(
      "Missing newline in assignment",
      lsp::Range::at(0, 12, 0, 12),
    )
    .warning(
      "Variable `baz` appears unused",
      lsp::Range::at(0, 13, 0, 16),
    )
    .run();

    AnalyzerTest::new(indoc! {
      "
      [private] foo:
      "
    })
    .error("Missing newline in attribute", lsp::Range::at(0, 9, 0, 9))
    .run();
  }

  #[test]
  fn parser_errors_invalid() {
    AnalyzerTest::new(indoc! {
      "
      foo
        echo \"foo\"
      "
    })
    .error(
      "Syntax error near `foo echo \"foo\"`",
      lsp::Range::at(0, 0, 2, 0),
    )
    .run();
  }

  #[test]
  fn parser_errors_invalid_attribute_on_wrapped_assignment() {
    AnalyzerTest::new(indoc! {
      "
      export [private]
      foo := 'bar'

      bar:
        echo {{foo}}
      "
    })
    .error("Syntax error near `export`", lsp::Range::at(0, 0, 0, 6))
    .run();

    AnalyzerTest::new(indoc! {
      "
      eager [private]
      foo := 'bar'

      bar:
        echo {{foo}}
      "
    })
    .error("Syntax error near `eager`", lsp::Range::at(0, 0, 0, 5))
    .run();
  }

  #[test]
  fn parser_errors_invalid_chained_comparison() {
    AnalyzerTest::new(indoc! {
      "
      set lists
      set unstable

      foo:
        echo {{ if 'a' == 'b' != 'c' { 'd' } else { 'e' } }}
      "
    })
    .error(
      "Expected `&&`, `||`, `+`, `++`, or `/`, but found `!=`",
      lsp::Range::at(4, 24, 4, 26),
    )
    .run();
  }

  #[test]
  fn parser_errors_invalid_double_braces_in_recipe() {
    AnalyzerTest::new(indoc! {
      r#"
      foo:
        echo "{{.Foo}}"
      "#
    })
    .error("Syntax error near `{{`", lsp::Range::at(1, 8, 1, 10))
    .run();
  }

  #[test]
  fn parser_errors_invalid_if_expression_operand() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo {{ if 'a' == 'b' { 'c' } else { 'd' } + 'e' }}
        echo {{ 'a' + if 'a' == 'b' { 'c' } else { 'd' } / 'e' }}
      "
    })
    .error(
      "Expected `&&`, `!=`, `!~`, `||`, `==`, or `=~`, but found `+`",
      lsp::Range::at(1, 45, 1, 46),
    )
    .error(
      "Expected `&&`, `!=`, `!~`, `||`, `==`, or `=~`, but found `/`",
      lsp::Range::at(2, 51, 2, 52),
    )
    .run();
  }

  #[test]
  fn parser_errors_invalid_numeric_literal() {
    AnalyzerTest::new(indoc! {
      "
      foo bar=1:
        echo {{bar}}
      "
    })
    .error("Unknown start of token `1`", lsp::Range::at(0, 8, 0, 9))
    .run();
  }

  #[test]
  fn parser_errors_valid() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn parser_errors_valid_with_bare_carriage_returns() {
    AnalyzerTest::new("foo := 'bar\rbaz' + qux\r\nbar := '🧪\rqux' + foo\n")
      .error("Variable `qux` not found", lsp::Range::at(1, 7, 1, 10))
      .warning("Variable `bar` appears unused", lsp::Range::at(2, 0, 2, 3))
      .run();
  }

  #[test]
  fn parser_errors_valid_with_double_braces_in_backticks() {
    AnalyzerTest::new(indoc! {
      r#"
      foo := `echo "{{.Foo}}"`

      bar baz=`echo "{{.Baz}}"` qux=```echo "{{.Qux}}"```:
        echo {{ foo }} {{ baz }} {{ qux }}
      "#
    })
    .run();
  }

  #[test]
  fn parser_errors_valid_with_escaped_braces_in_format_strings() {
    AnalyzerTest::new(indoc! {
      r#"
      name := "world"
      foo := f'{{{{'
      bar := f"{{{{ {{name}}"
      baz := f'''{{{{'''
      qux := f"""{{{{ {{name}}"""

      recipe:
        echo {{ foo }} {{ bar }} {{ baz }} {{ qux }}
      "#
    })
    .run();
  }

  #[test]
  fn parser_errors_valid_with_guard_prefixes() {
    AnalyzerTest::new(indoc! {
      "
      set guards

      foo:
        ?guard
        @?quiet_guard
        ?@guard_quiet
      "
    })
    .run();
  }

  #[test]
  fn parser_errors_valid_with_quotes_in_indented_strings() {
    #[track_caller]
    fn case(string: &str) {
      AnalyzerTest::new(&format!(
        "foo := {string}\n\nbar:\n  echo {{{{foo}}}}\n"
      ))
      .run();
    }

    case(r#""""foo""bar""""#);
    case(r#"x"""foo""bar""""#);
    case(r#"f"""foo""bar""""#);
    case("f'''foo''bar'''");
  }

  #[test]
  fn parser_errors_valid_with_list_expressions() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      foo := ['foo'] ++ ['bar']
      bar := 'foo' == 'foo'
      baz := 'foo' =~ '^f'

      qux:
        echo {{ foo }}
        echo {{ bar }}
        echo {{ baz }}
      "
    })
    .run();
  }

  #[test]
  fn parser_errors_valid_with_multiline_recipe_interpolations() {
    #[track_caller]
    fn case(expression: &str) {
      for prefix in ["bar ", "{{'bar'}}", "{{{{ "] {
        AnalyzerTest::new(&format!(
          "foo:\n  {prefix}{{{{{expression}}}}} qux\n  quux\n"
        ))
        .run();
      }
    }

    case("'\nbaz\n'");
    case("\"\nbaz\n\"");
    case("`\nbaz\n`");
  }

  #[test]
  fn parser_errors_valid_with_recipe_line_containing_only_open_brace() {
    AnalyzerTest::new(indoc! {
      r#"
      foo bar="baz" qux="quux":
        #!/usr/bin/env bash
        cat <<'JSON'
        {
          "foo": "bar",
          "{{ bar }}": "{{ qux }}"
        }
        JSON
      "#
    })
    .run();
  }

  #[test]
  fn parser_errors_valid_with_shebang_interpolation() {
    AnalyzerTest::new(indoc! {
      "
      foo bar:
        #!/usr/bin/env {{bar}}
      "
    })
    .run();
  }

  #[test]
  fn parser_errors_valid_with_shebang_operands() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        #!cmd /c
        echo bar
      "
    })
    .run();
  }

  #[test]
  fn parser_errors_valid_with_shell_expanded_strings() {
    AnalyzerTest::new(indoc! {
      r#"
      import? x'~/.config/just/common.just'

      greeting := x"~/$USER/${GREETING:-hello}"

      foo:
        echo {{greeting}}
      "#
    })
    .run();
  }

  #[test]
  fn parser_errors_valid_with_slash_prefixed_operands() {
    AnalyzerTest::new(indoc! {
      "
      foo := 'bar' + / 'baz'

      bar:
        echo {{foo}}
      "
    })
    .run();
  }

  #[test]
  fn recipe_dependencies_repeated_subsequent_marker() {
    AnalyzerTest::new(indoc! {
      "
      foo: && bar && baz && qux

      bar:

      baz:

      qux:
      "
    })
    .error(
      "Expected comment, end of file, end of line, or dependency, but found `&&`",
      lsp::Range::at(0, 12, 0, 14),
    )
    .error(
      "Expected comment, end of file, end of line, or dependency, but found `&&`",
      lsp::Range::at(0, 19, 0, 21),
    )
    .run();
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
