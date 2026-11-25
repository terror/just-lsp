use super::*;

const COMPARISON_OPERATORS: &[&str] = &["!=", "!~", "==", "=~"];
const LOGICAL_OPERATORS: &[&str] = &["&&", "||"];

define_rule! {
  ListFeatureGateRule {
    id: "list-feature-gate",
    message: "list feature requires set lists",
  }
}

impl ListFeatureGateRule {
  fn comparison_operator(node: Node<'_>) -> Option<Node<'_>> {
    Self::operator(node, COMPARISON_OPERATORS)
  }

  fn condition_comparison_operator(node: Node<'_>) -> Option<Node<'_>> {
    (0..node.child_count())
      .filter_map(|index| node.child(index))
      .find(|child| child.kind() == "expression")
      .and_then(Self::comparison_operator)
  }

  fn function_message(name: &str) -> Option<&'static str> {
    match name {
      "bool" => Some("the `bool()` function requires `set lists`"),
      "join_list" => Some("the `join_list()` function requires `set lists`"),
      "num_jobs" => Some("the `num_jobs()` function requires `set lists`"),
      "show" => Some("the `show()` function requires `set lists`"),
      "split" => Some("the `split()` function requires `set lists`"),
      "which" => Some("the `which()` function requires `set lists`"),
      _ => None,
    }
  }

  fn if_token(node: Node<'_>) -> Option<Node<'_>> {
    (0..node.child_count())
      .filter_map(|index| node.child(index))
      .find(|child| child.kind() == "if")
  }

  fn interpreter_setting_array(document: &Document, node: Node<'_>) -> bool {
    let Some(setting) = node.get_parent("setting") else {
      return false;
    };

    let Some(array) = setting.find("list_literal") else {
      return false;
    };

    let Some(name) = setting.child_by_field_name("left") else {
      return false;
    };

    if array.start_byte() != node.start_byte()
      || array.end_byte() != node.end_byte()
    {
      return false;
    }

    matches!(
      document.get_node_text(&name).as_str(),
      "script-interpreter" | "shell" | "windows-shell"
    )
  }

  fn operator<'tree>(
    node: Node<'tree>,
    operators: &[&str],
  ) -> Option<Node<'tree>> {
    (0..node.child_count())
      .filter_map(|index| node.child(index))
      .find(|child| operators.contains(&child.kind()))
  }

  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    if context.setting_enabled("lists") {
      return diagnostics;
    }

    diagnostics.extend(
      context
        .attributes()
        .iter()
        .filter(|attribute| attribute.name.value == "arg")
        .flat_map(|attribute| &attribute.arguments)
        .filter_map(|argument| match argument {
          AttributeArgument::Keyword { name, .. } => {
            let message = match name.value.as_str() {
              "flag" => "`flag` arguments require `set lists`",
              "max" => "`[arg(max)]` requires `set lists`",
              "min" => "`[arg(min)]` requires `set lists`",
              "multiple" => "`[arg(multiple)]` requires `set lists`",
              _ => return None,
            };

            Some(Diagnostic::error(message, name.range))
          }
          AttributeArgument::Positional(_) => None,
        }),
    );

    let _ = TreeWalker::new(context.tree().root_node()).walk(|node, _| {
      Self::validate_node(context, context.document(), node, &mut diagnostics);
      ControlFlow::<(), bool>::Continue(true)
    });

    diagnostics
  }

  fn validate_node(
    context: &RuleContext<'_>,
    document: &Document,
    node: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
  ) {
    match node.kind() {
      "condition" => {
        if Self::condition_comparison_operator(node).is_none() {
          diagnostics.push(Diagnostic::error(
            "`if` and `assert` conditions other than comparisons require `set lists`",
            document.get_range(&node),
          ));
        }
      }
      "expression" => {
        if let Some(operator) = Self::operator(node, LOGICAL_OPERATORS) {
          diagnostics.push(Diagnostic::error(
            "logical operators require `set lists`",
            document.get_range(&operator),
          ));
        }

        if let Some(operator) = Self::operator(node, &["++"]) {
          diagnostics.push(Diagnostic::error(
            "list concatenation operator `++` requires `set lists`",
            document.get_range(&operator),
          ));
        }

        if node
          .parent()
          .is_none_or(|parent| parent.kind() != "condition")
          && let Some(operator) = Self::comparison_operator(node)
        {
          diagnostics.push(Diagnostic::error(
            "comparison operators require `set lists`",
            document.get_range(&operator),
          ));
        }
      }
      "function_call" => {
        if let Some(name) = node.child_by_field_name("name") {
          let name_text = document.get_node_text(&name);

          if !context.user_function_names().contains(&name_text)
            && let Some(message) = Self::function_message(&name_text)
          {
            diagnostics
              .push(Diagnostic::error(message, document.get_range(&name)));
          }
        }
      }
      "if_expression" => {
        if node.find("^else_clause").is_none() {
          diagnostics.push(Diagnostic::error(
            "`if` without `else` requires `set lists`",
            document.get_range(&Self::if_token(node).unwrap_or(node)),
          ));
        }
      }
      "list_literal" => {
        if !Self::interpreter_setting_array(document, node) {
          diagnostics.push(Diagnostic::error(
            "list literals require `set lists`",
            document.get_range(&node),
          ));
        }
      }
      "not_expression" => {
        if let Some(operator) = Self::operator(node, &["!"]) {
          diagnostics.push(Diagnostic::error(
            "negation operator requires `set lists`",
            document.get_range(&operator),
          ));
        }
      }
      _ => {}
    }
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::{formatdoc, indoc},
  };

  #[test]
  fn accepts_logical_operators_with_lists() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      foo := '' || 'bar'
      bar := 'foo' && 'bar'

      baz:
        echo {{ foo }}
        echo {{ bar }}
      "
    })
    .run();
  }

  #[test]
  fn accepts_unary_negation_with_lists() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      foo bar:
        echo {{ !bar }}
      "
    })
    .run();
  }

  #[test]
  fn arg_attribute_with_flag_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      [arg('foo', long, flag)]
      bar foo:
        echo {{foo}}
      "
    })
    .error(
      "`flag` arguments require `set lists`",
      lsp::Range::at(0, 18, 0, 22),
    )
    .run();
  }

  #[test]
  fn arg_keywords_require_lists() {
    #[track_caller]
    fn case(arguments: &str, parameter: &str, message: &'static str, end: u32) {
      let content = formatdoc! {
        "
        [arg('foo', {arguments})]
        bar {parameter}:
          echo {{{{foo}}}}
        "
      };

      let range = lsp::Range::at(0, 12, 0, end);

      AnalyzerTest::new(&content).error(message, range).run();

      AnalyzerTest::new(&format!(
        "{content}\nset unstable\nset lists := false\n"
      ))
      .error(message, range)
      .run();

      AnalyzerTest::new(&format!("{content}import 'foo.just'\n"))
        .file("foo.just", "set lists := false\n")
        .error(message, range)
        .run();

      AnalyzerTest::new(&format!("{content}import 'foo.just'\n"))
        .file("foo.just", "set unstable\nset lists\n")
        .run();
    }

    case("max='2'", "+foo", "`[arg(max)]` requires `set lists`", 15);
    case("min='1'", "+foo", "`[arg(min)]` requires `set lists`", 15);
    case(
      "multiple, long",
      "foo",
      "`[arg(multiple)]` requires `set lists`",
      20,
    );
  }

  #[test]
  fn assert_accepts_condition_operators() {
    AnalyzerTest::new(indoc! {
      "
      foo name:
        {{ assert(name == \"bar\") }}
        {{ assert(name == \"bar\", \"msg\") }}
        {{ assert(name != \"bar\", \"msg\") }}
        {{ assert(name =~ \"^[a-z]+$\", \"msg\") }}
        {{ assert(name !~ \"^[a-z]+$\", \"msg\") }}
      "
    })
    .run();
  }

  #[test]
  fn list_feature_gate_allow_shadowed_functions_without_lists() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      bool(value) := value
      join_list(value) := value
      len(value) := value
      num_jobs() := 'qux'
      show(value) := value
      split(value) := value
      which(value) := value

      foo := bool('foo')
      bar := join_list('bar')
      baz := len('baz')
      qux := num_jobs()
      quux := show('quux')
      corge := split('corge')
      grault := which('grault')

      recipe:
        echo {{ foo }}
        echo {{ bar }}
        echo {{ baz }}
        echo {{ qux }}
        echo {{ quux }}
        echo {{ corge }}
        echo {{ grault }}
      "
    })
    .run();
  }

  #[test]
  fn list_feature_gate_bool_function_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo := bool('true')

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "the `bool()` function requires `set lists`",
      lsp::Range::at(0, 7, 0, 11),
    )
    .run();
  }

  #[test]
  fn list_feature_gate_comparison_as_value_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo := 'a' == 'b'

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "comparison operators require `set lists`",
      lsp::Range::at(0, 11, 0, 13),
    )
    .run();
  }

  #[test]
  fn list_feature_gate_if_without_else_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo := if 'a' == 'b' { 'c' }

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "`if` without `else` requires `set lists`",
      lsp::Range::at(0, 7, 0, 9),
    )
    .run();
  }

  #[test]
  fn list_feature_gate_join_list_function_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo := join_list('bar')

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "the `join_list()` function requires `set lists`",
      lsp::Range::at(0, 7, 0, 16),
    )
    .run();
  }

  #[test]
  fn list_feature_gate_len_function_accepts_scalar_without_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo := len('bar')

      bar:
        echo {{ foo }}
      "
    })
    .run();
  }

  #[test]
  fn list_feature_gate_list_concatenation_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo := 'a' ++ 'b'

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "list concatenation operator `++` requires `set lists`",
      lsp::Range::at(0, 11, 0, 13),
    )
    .run();
  }

  #[test]
  fn list_feature_gate_list_literals_require_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo := ['bar']

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "list literals require `set lists`",
      lsp::Range::at(0, 7, 0, 14),
    )
    .run();
  }

  #[test]
  fn list_feature_gate_logical_and_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo := 'foo' && 'bar'

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "logical operators require `set lists`",
      lsp::Range::at(0, 13, 0, 15),
    )
    .run();
  }

  #[test]
  fn list_feature_gate_logical_or_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo := '' || 'bar'

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "logical operators require `set lists`",
      lsp::Range::at(0, 10, 0, 12),
    )
    .run();
  }

  #[test]
  fn list_feature_gate_negation_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo bar:
        echo {{ !bar }}
      "
    })
    .error(
      "negation operator requires `set lists`",
      lsp::Range::at(1, 10, 1, 11),
    )
    .run();
  }

  #[test]
  fn list_feature_gate_non_comparison_assert_condition_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo {{ assert('bar') }}
      "
    })
    .error(
      "`if` and `assert` conditions other than comparisons require `set lists`",
      lsp::Range::at(1, 17, 1, 22),
    )
    .run();
  }

  #[test]
  fn list_feature_gate_non_comparison_if_condition_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo := if 'a' { 'b' } else { 'c' }

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "`if` and `assert` conditions other than comparisons require `set lists`",
      lsp::Range::at(0, 10, 0, 13),
    )
    .run();
  }

  #[test]
  fn list_feature_gate_num_jobs_function_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo := num_jobs()

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "the `num_jobs()` function requires `set lists`",
      lsp::Range::at(0, 7, 0, 15),
    )
    .run();
  }

  #[test]
  fn list_feature_gate_show_function_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo := show('bar')

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "the `show()` function requires `set lists`",
      lsp::Range::at(0, 7, 0, 11),
    )
    .run();
  }

  #[test]
  fn list_feature_gate_split_function_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo := split('bar')

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "the `split()` function requires `set lists`",
      lsp::Range::at(0, 7, 0, 12),
    )
    .run();
  }

  #[test]
  fn list_feature_gate_which_function_requires_lists() {
    AnalyzerTest::new(indoc! {
      "
      foo := which('bar')

      bar:
        echo {{ foo }}
      "
    })
    .error(
      "the `which()` function requires `set lists`",
      lsp::Range::at(0, 7, 0, 12),
    )
    .run();
  }

  #[test]
  fn settings_dotenv_lists_require_lists() {
    #[track_caller]
    fn case(setting: &str, range: lsp::Range) {
      AnalyzerTest::new(&formatdoc! {
        r#"
        set {setting} := ["foo", "bar"]

        foo:
          echo "foo"
        "#
      })
      .error("list literals require `set lists`", range)
      .run();
    }

    case("dotenv-filename", lsp::Range::at(0, 23, 0, 37));
    case("dotenv-path", lsp::Range::at(0, 19, 0, 33));
    case("dotenv-command", lsp::Range::at(0, 22, 0, 36));
  }
}
