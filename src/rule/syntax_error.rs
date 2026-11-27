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
    let text = document.get_node_text(node);

    let preview = SyntaxErrorRule::snippet_preview(&text);

    if text == "\r" {
      "Unpaired carriage return".to_string()
    } else if let Some(snippet) = preview {
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

  fn expected_tokens(
    document: &Document,
    node: Node<'_>,
    first: bool,
  ) -> Option<Vec<&'static str>> {
    let parent = node.parent()?;

    if first && node.kind() == "@" {
      return Some(vec!["identifier"]);
    }

    if first
      || parent.kind() == "recipe_header"
        && parent.child_by_field_name("name") == Some(node)
    {
      return Some(vec!["`*`", "`:`", "`$`", "identifier", "`+`"]);
    }

    let expression = vec![
      "backtick",
      "`!`",
      "`[`",
      "identifier",
      "`(`",
      "`/`",
      "string",
    ];

    let parameter = vec!["`*`", "`:`", "`$`", "`=`", "identifier", "`+`"];

    match (node.kind(), parent.kind()) {
      ("$" | "::", _) | (":=", "alias") | ("?", "module") => {
        return Some(vec!["identifier"]);
      }
      ("*" | "+", "variadic_parameter") => {
        return Some(vec!["`$`", "identifier"]);
      }
      ("*", "dependency_expression") | ("identifier", "assert_expression") => {
        return Some(vec!["`(`"]);
      }
      ("!", _) | ("=", "parameter") => {
        return Some(vec![
          "backtick",
          "`!`",
          "`[`",
          "identifier",
          "`(`",
          "string",
        ]);
      }
      ("?", "import") => return Some(vec!["identifier", "string"]),
      (")", "function_definition") => return Some(vec!["`:=`"]),
      (":=", "setting") => {
        let name = parent.child_by_field_name("left")?;

        return Some(match Builtin::setting(&document.get_node_text(&name)) {
          Some(Builtin::Setting {
            kind: SettingKind::Array,
            ..
          }) => vec!["`[`"],
          Some(Builtin::Setting {
            kind: SettingKind::Boolean(_),
            ..
          }) => vec!["identifier"],
          _ => expression,
        });
      }
      ("else", _) => return Some(vec!["`{`"]),
      (
        "!=" | "!~" | "&&" | "+" | "++" | "/" | ":=" | "==" | "=~" | "if"
        | "||",
        _,
      ) => return Some(expression),
      ("identifier", "alias")
        if parent.child_by_field_name("left") == Some(node) =>
      {
        return Some(parameter);
      }
      ("identifier", "assignment")
        if parent.parent().is_some_and(|grandparent| {
          matches!(grandparent.kind(), "eager" | "export")
        }) && parent.child_by_field_name("left") == Some(node) =>
      {
        return Some(parameter);
      }
      ("identifier", "parameter")
        if parent.child_by_field_name("name") == Some(node) =>
      {
        return Some(
          if parent.parent().is_some_and(|grandparent| {
            grandparent.kind() == "variadic_parameter"
          }) {
            vec!["`:`", "`=`"]
          } else {
            parameter
          },
        );
      }
      _ => {}
    }

    let variable = node.kind() == "identifier"
      && (parent.kind() == "value"
        || parent.kind() == "function_call"
          && parent.child_by_field_name("name") == Some(node));

    let context = successors(Some(parent), Node::parent).find(|ancestor| {
      ancestor.kind() == "parameter"
        || ancestor.kind() == "condition"
          && ancestor.parent().is_some_and(|parent| {
            matches!(parent.kind(), "else_if_clause" | "if_expression")
          })
    })?;

    if context.kind() == "parameter" {
      let variadic = context
        .parent()
        .is_some_and(|parent| parent.kind() == "variadic_parameter");

      return Some(
        [
          (!variadic).then_some("`*`"),
          Some("`:`"),
          (!variadic).then_some("`$`"),
          (!variadic).then_some("identifier"),
          variable.then_some("`(`"),
          (!variadic).then_some("`+`"),
        ]
        .into_iter()
        .flatten()
        .collect(),
      );
    }

    let comparison = successors(Some(parent), Node::parent)
      .take_while(|ancestor| *ancestor != context)
      .any(|ancestor| {
        let mut cursor = ancestor.walk();

        ancestor.children(&mut cursor).any(|child| {
          matches!(child.kind(), "!=" | "!~" | "==" | "=~")
            && child.start_byte() < node.start_byte()
        })
      });

    Some(
      [
        Some("`&&`"),
        (!comparison).then_some("`!=`"),
        (!comparison).then_some("`!~`"),
        Some("`||`"),
        Some("`{`"),
        (!comparison).then_some("`==`"),
        (!comparison).then_some("`=~`"),
        variable.then_some("`(`"),
        Some("`+`"),
        Some("`++`"),
        Some("`/`"),
      ]
      .into_iter()
      .flatten()
      .collect(),
    )
  }

  fn indented_items(document: &Document, root: Node<'_>) -> Vec<Diagnostic> {
    let mut cursor = root.walk();

    root
      .children(&mut cursor)
      .filter(|item| !item.is_error())
      .flat_map(|item| {
        let mut cursor = item.walk();

        let after_attributes = item
          .children(&mut cursor)
          .filter(|child| child.kind() == "attribute")
          .filter_map(|attribute| attribute.next_sibling())
          .map(|node| (node, true))
          .collect::<Vec<_>>();

        once((item, false)).chain(after_attributes)
      })
      .filter_map(|(node, after_attribute)| {
        let (start, column) = (node.start_byte(), node.start_position().column);

        let line_start = start - column;

        let indented = column > 0
          && document
            .content
            .byte_slice(line_start..start)
            .bytes()
            .all(|byte| byte == b' ' || byte == b'\t');

        indented.then(|| {
          Diagnostic::new(
            if after_attribute {
              "Expected `@`, `[`, comment, end of line, or identifier, but found indent"
            } else {
              "Expected `@`, `[`, comment, end of file, end of line, or identifier, but found indent"
            },
            lsp::Range {
              start: document.content.byte_to_lsp_position(line_start),
              end: document.content.byte_to_lsp_position(start),
            },
          )
        })
      })
      .collect()
  }

  fn leaves<'a>(node: Node<'a>, leaves: &mut Vec<Node<'a>>) {
    if node.child_count() == 0
      || matches!(node.kind(), "external_command" | "string")
    {
      leaves.push(node);
      return;
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
      SyntaxErrorRule::leaves(child, leaves);
    }
  }

  fn line_break(
    document: &Document,
    leaves: &[Node<'_>],
  ) -> Option<Diagnostic> {
    let start = leaves.iter().position(|leaf| leaf.kind() != "comment")?;

    let leaves = &leaves[start..];

    let column = leaves[0].start_position().column;

    if column > 0
      && document
        .content
        .byte_slice(leaves[0].start_byte() - column..leaves[0].start_byte())
        .bytes()
        .all(|byte| byte == b' ' || byte == b'\t')
    {
      return None;
    }

    let mut depth = 0usize;

    for (index, pair) in leaves.windows(2).enumerate() {
      let (previous, next) = (pair[0], pair[1]);

      match previous.kind() {
        "(" | "[" | "{" => depth += 1,
        ")" | "]" | "}" => depth = depth.saturating_sub(1),
        _ => {}
      }

      if depth > 0 || previous.kind() == "comment" {
        continue;
      }

      let gap = document
        .content
        .byte_slice(previous.end_byte()..next.start_byte())
        .bytes()
        .collect::<Vec<u8>>();

      let newline = gap.iter().enumerate().find_map(|(i, byte)| {
        if *byte != b'\n' {
          return None;
        }

        let eol = if i > 0 && gap[i - 1] == b'\r' {
          i - 1
        } else {
          i
        };

        (eol == 0 || gap[eol - 1] != b'\\').then_some(previous.end_byte() + eol)
      });

      let (found, range) = if let Some(newline) = newline {
        let position = document.content.byte_to_lsp_position(newline);

        (
          "end of line",
          lsp::Range {
            start: position,
            end: position,
          },
        )
      } else if next.kind() == "comment" && index + 2 < leaves.len() {
        ("comment", document.get_range(&next))
      } else {
        continue;
      };

      let expected =
        SyntaxErrorRule::expected_tokens(document, previous, index == 0)?;

      return Some(Diagnostic::new(
        format!("Expected {}, but found {found}", List::or(expected)),
        range,
      ));
    }

    None
  }

  fn line_breaks(document: &Document, root: Node<'_>) -> Vec<Diagnostic> {
    let mut cursor = root.walk();

    root
      .children(&mut cursor)
      .filter(|item| item.kind() != "shebang" && !item.has_error())
      .filter_map(|item| {
        let mut cursor = item.walk();

        let mut leaves = Vec::new();

        for child in item.children(&mut cursor).filter(|child| {
          if item.kind() == "recipe" {
            child.kind() == "recipe_header"
          } else {
            child.kind() != "attribute"
          }
        }) {
          SyntaxErrorRule::leaves(child, &mut leaves);
        }

        SyntaxErrorRule::line_break(document, &leaves)
      })
      .collect()
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

        return ControlFlow::Continue(false);
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

      if node.kind() == "dependencies"
        && let Some(marker) = node
          .child(node.child_count().saturating_sub(1))
          .filter(|child| child.kind() == "&&")
      {
        let end = document.get_range(&marker).end;

        diagnostics.push(Diagnostic::new(
          "Expected `*`, identifier, or `(`, but found end of line",
          lsp::Range { start: end, end },
        ));
      }

      if node.is_missing() {
        diagnostics.push(Diagnostic::new(
          SyntaxErrorRule::missing_message(&node),
          document.get_range(&node),
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

      let node = if node == root {
        SyntaxErrorRule::unterminated_item(root).unwrap_or(root)
      } else {
        node
      };

      let end = document.get_range(&node).end;

      diagnostics.push(Diagnostic::new(
        format!(
          "Missing newline in {}",
          SyntaxErrorRule::describe_kind(node.kind())
        ),
        lsp::Range { start: end, end },
      ));
    }

    diagnostics.extend(SyntaxErrorRule::indented_items(document, root));

    diagnostics.extend(SyntaxErrorRule::line_breaks(document, root));

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

  fn unterminated_item(root: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = root.walk();

    let children = root.children(&mut cursor).collect::<Vec<_>>();

    children
      .windows(2)
      .find(|pair| {
        pair[1].kind() != "comment"
          && pair[0].end_position().row == pair[1].start_position().row
      })
      .map(|pair| pair[0])
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
      "Syntax error near `,`",
      lsp::Range::at(0, 5, 0, 6),
    );
    case(
      "[arg('bar', short,,)]",
      "Syntax error near `,`",
      lsp::Range::at(0, 18, 0, 19),
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
  fn parser_errors_invalid_after_alias_or_import() {
    #[track_caller]
    fn case(source: &str, message: &'static str, range: lsp::Range) {
      AnalyzerTest::new(source).error(message, range).run();
    }

    case(
      "alias foo := bar bar:\n",
      "Missing newline in alias",
      lsp::Range::at(0, 16, 0, 16),
    );
    case(
      "import? 'foo' bar:\n",
      "Missing newline in import",
      lsp::Range::at(0, 13, 0, 13),
    );
  }

  #[test]
  fn parser_errors_invalid_assert_trailing_comma() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo {{ assert('bar' == 'bar', 'baz',) }}
      "
    })
    .error("Syntax error near `,`", lsp::Range::at(1, 38, 1, 39))
    .run();
  }

  #[test]
  fn parser_errors_invalid_brace_runs_in_recipe() {
    AnalyzerTest::new(indoc! {
      "
      foo bar:
        echo {{{bar}}
        echo {{{{{{{bar}}
      "
    })
    .error("Syntax error near `{`", lsp::Range::at(1, 9, 1, 10))
    .error("Syntax error near `{`", lsp::Range::at(2, 13, 2, 14))
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
    .error("Syntax error near `!= 'c'`", lsp::Range::at(4, 24, 4, 30))
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
    .error("Syntax error near `.`", lsp::Range::at(1, 10, 1, 11))
    .error("Variable `Foo` not found", lsp::Range::at(1, 11, 1, 14))
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
  fn parser_errors_invalid_indented_items() {
    #[track_caller]
    fn case(source: &str, message: &'static str, range: lsp::Range) {
      AnalyzerTest::new(source).error(message, range).run();
    }

    case(
      "  foo:\n",
      "Expected `@`, `[`, comment, end of file, end of line, or identifier, but found indent",
      lsp::Range::at(0, 0, 0, 2),
    );
    case(
      "\t# foo\n",
      "Expected `@`, `[`, comment, end of file, end of line, or identifier, but found indent",
      lsp::Range::at(0, 0, 0, 1),
    );
    case(
      "  foo\n:\n",
      "Expected `@`, `[`, comment, end of file, end of line, or identifier, but found indent",
      lsp::Range::at(0, 0, 0, 2),
    );
    case(
      "[private]\n  foo:\n",
      "Expected `@`, `[`, comment, end of line, or identifier, but found indent",
      lsp::Range::at(1, 0, 1, 2),
    );
  }

  #[test]
  fn parser_errors_invalid_line_breaks() {
    #[track_caller]
    fn case(source: &str, message: &'static str, range: lsp::Range) {
      AnalyzerTest::new(source).error(message, range).run();
    }

    case(
      "foo\n:\n",
      "Expected `*`, `:`, `$`, identifier, or `+`, but found end of line",
      lsp::Range::at(0, 3, 0, 3),
    );
    case(
      "foo # bar\n:\n",
      "Expected `*`, `:`, `$`, identifier, or `+`, but found comment",
      lsp::Range::at(0, 4, 0, 9),
    );
    case(
      "foo \\\n\n:\n",
      "Expected `*`, `:`, `$`, identifier, or `+`, but found end of line",
      lsp::Range::at(1, 0, 1, 0),
    );
    case(
      "export foo\n:= 'bar'\n",
      "Expected `*`, `:`, `$`, `=`, identifier, or `+`, but found end of line",
      lsp::Range::at(0, 10, 0, 10),
    );
    case(
      "foo $bar=\n'baz':\n",
      "Expected backtick, `!`, `[`, identifier, `(`, or string, but found end of line",
      lsp::Range::at(0, 9, 0, 9),
    );
    case(
      "baz := 'qux'\n\nfoo +$bar=baz\n:\n",
      "Expected `:` or `(`, but found end of line",
      lsp::Range::at(2, 13, 2, 13),
    );
    case(
      "set quiet :=\ntrue\n",
      "Expected identifier, but found end of line",
      lsp::Range::at(0, 12, 0, 12),
    );
    case(
      "export foo := if 'a' == 'b'\n{ 'c' } else { 'd' }\n",
      "Expected `&&`, `||`, `{`, `+`, `++`, or `/`, but found end of line",
      lsp::Range::at(0, 27, 0, 27),
    );
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
  fn parser_errors_invalid_recovers_recipe_body() {
    AnalyzerTest::new(indoc! {
      "
      foo := 'bar' }

      baz:
        echo {{foo}}
      "
    })
    .error("Syntax error near `}`", lsp::Range::at(0, 13, 0, 14))
    .run();
  }

  #[test]
  fn parser_errors_invalid_unpaired_carriage_return() {
    #[track_caller]
    fn case(source: &str, message: &'static str, range: lsp::Range) {
      AnalyzerTest::new(source).error(message, range).run();
    }

    case(
      "foo:\r",
      "Syntax error near `foo:`",
      lsp::Range::at(0, 0, 1, 0),
    );
    case(
      "foo: \rbar\n\nbar:\n",
      "Unpaired carriage return",
      lsp::Range::at(0, 5, 1, 0),
    );
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
  fn parser_errors_valid_with_line_breaks_in_delimiters() {
    #[track_caller]
    fn case(source: &str) {
      AnalyzerTest::new(source).run();
    }

    case("export foo := 'bar' \\\n+ 'baz'\n");
    case("export foo := ( # bar\n'baz')\n");
    case("[group(\n'foo'\n)]\nbar:\n");
  }

  #[test]
  fn parser_errors_valid_with_line_continuation_before_blank_line() {
    #[track_caller]
    fn case(source: &str) {
      AnalyzerTest::new(source).run();
    }

    case("foo := 'bar' \\\n\nbaz:\n  echo {{foo}}\n");
    case("baz:\n  echo {{foo}}\n\nfoo := 'bar' \\\n");
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
  fn parser_errors_valid_with_nul_bytes() {
    #[track_caller]
    fn case(source: &str) {
      AnalyzerTest::new(source).run();
    }

    case("# foo\0bar\n");
    case("foo := 'bar\0baz'\n\nqux:\n  echo {{foo}}\n");
    case("foo := `bar\0baz`\n\nqux:\n  echo {{foo}}\n");
    case("foo:\n  #!/bin/sh bar\0baz\n");
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
  fn parser_errors_valid_with_shebang_without_language() {
    #[track_caller]
    fn case(shebang: &str) {
      AnalyzerTest::new(&format!("foo:\n  {shebang}\n  echo bar\n")).run();
    }

    case("#!/usr/bin/env` python");
    case("#!/usr/ + bin/env python");
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
  fn parser_errors_valid_with_sigil_only_lines() {
    AnalyzerTest::new(indoc! {
      "
      set guards

      foo:
        @
        -
        ?
        @-
      "
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
  fn recipe_dependencies_marker_without_dependency() {
    #[track_caller]
    fn case(source: &str, range: lsp::Range) {
      AnalyzerTest::new(source)
        .error(
          "Expected `*`, identifier, or `(`, but found end of line",
          range,
        )
        .run();
    }

    case("foo: &&\n  echo\n", lsp::Range::at(0, 7, 0, 7));
    case("foo: bar &&\n\nbar:\n", lsp::Range::at(0, 11, 0, 11));
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
    .error("Syntax error near `&&`", lsp::Range::at(0, 12, 0, 14))
    .error("Syntax error near `&&`", lsp::Range::at(0, 19, 0, 21))
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
