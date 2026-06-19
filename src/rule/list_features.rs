use super::*;

const COMPARISON_OPERATORS: &[&str] = &["!=", "!~", "==", "=~"];
const LOGICAL_OPERATORS: &[&str] = &["&&", "||"];

define_rule! {
  ListFeaturesRule {
    id: "list-features",
    message: "list feature requires set lists",
    run(context) {
      let mut diagnostics = Vec::new();

      if context.setting_enabled("lists") {
        return diagnostics;
      }

      let Some(tree) = context.tree() else {
        return diagnostics;
      };

      Self::validate_node(
        context,
        context.document(),
        tree.root_node(),
        &mut diagnostics,
      );

      diagnostics
    }
  }
}

impl ListFeaturesRule {
  fn arg_flag_range(document: &Document, node: Node<'_>) -> Option<lsp::Range> {
    let name = node.child_by_field_name("name")?;

    if document.get_node_text(&name) != "flag" {
      return None;
    }

    let mut sibling = node.prev_sibling();

    while let Some(node) = sibling {
      if node.kind() == "identifier" {
        return (document.get_node_text(&node) == "arg")
          .then(|| name.get_range(document));
      }

      sibling = node.prev_sibling();
    }

    None
  }

  fn comparison_operator(node: Node<'_>) -> Option<Node<'_>> {
    Self::operator(node, COMPARISON_OPERATORS)
  }

  fn condition_comparison_operator(node: Node<'_>) -> Option<Node<'_>> {
    (0..node.child_count())
      .filter_map(|index| node.child(index.try_into().ok()?))
      .find(|child| child.kind() == "expression")
      .and_then(Self::comparison_operator)
  }

  fn function_message(name: &str) -> Option<&'static str> {
    match name {
      "bool" => Some("the `bool()` function requires `set lists`"),
      "join_list" => Some("the `join_list()` function requires `set lists`"),
      "show" => Some("the `show()` function requires `set lists`"),
      "split" => Some("the `split()` function requires `set lists`"),
      "which" => Some("the `which()` function requires `set lists`"),
      _ => None,
    }
  }

  fn if_token(node: Node<'_>) -> Option<Node<'_>> {
    (0..node.child_count())
      .filter_map(|index| node.child(index.try_into().ok()?))
      .find(|child| child.kind() == "if")
  }

  fn interpreter_setting_array(document: &Document, node: Node<'_>) -> bool {
    let Some(setting) = node.get_parent("setting") else {
      return false;
    };

    let Some(array) = setting.find("list_literal") else {
      return false;
    };

    let Some(name) = setting.child(1) else {
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
      .filter_map(|index| node.child(index.try_into().ok()?))
      .find(|child| operators.contains(&child.kind()))
  }

  fn validate_node(
    context: &RuleContext<'_>,
    document: &Document,
    node: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
  ) {
    match node.kind() {
      "attribute_named_param" => {
        if let Some(range) = Self::arg_flag_range(document, node) {
          diagnostics.push(Diagnostic::error(
            "`flag` arguments require `set lists`",
            range,
          ));
        }
      }
      "condition" => {
        if Self::condition_comparison_operator(node).is_none() {
          diagnostics.push(Diagnostic::error(
            "`if` and `assert` conditions other than comparisons require `set lists`",
            node.get_range(document),
          ));
        }
      }
      "expression" => {
        if let Some(operator) = Self::operator(node, LOGICAL_OPERATORS) {
          diagnostics.push(Diagnostic::error(
            "logical operators require `set lists`",
            operator.get_range(document),
          ));
        }

        if let Some(operator) = Self::operator(node, &["++"]) {
          diagnostics.push(Diagnostic::error(
            "list concatenation operator `++` requires `set lists`",
            operator.get_range(document),
          ));
        }

        if node
          .parent()
          .is_none_or(|parent| parent.kind() != "condition")
          && let Some(operator) = Self::comparison_operator(node)
        {
          diagnostics.push(Diagnostic::error(
            "comparison operators require `set lists`",
            operator.get_range(document),
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
              .push(Diagnostic::error(message, name.get_range(document)));
          }
        }
      }
      "if_expression" => {
        if node.find("^else_clause").is_none() {
          diagnostics.push(Diagnostic::error(
            "`if` without `else` requires `set lists`",
            Self::if_token(node).unwrap_or(node).get_range(document),
          ));
        }
      }
      "list_literal" => {
        if !Self::interpreter_setting_array(document, node) {
          diagnostics.push(Diagnostic::error(
            "list literals require `set lists`",
            node.get_range(document),
          ));
        }
      }
      "not_expression" => {
        if let Some(operator) = Self::operator(node, &["!"]) {
          diagnostics.push(Diagnostic::error(
            "negation operator requires `set lists`",
            operator.get_range(document),
          ));
        }
      }
      _ => {}
    }

    for index in 0..node.child_count() {
      if let Ok(index) = index.try_into()
        && let Some(child) = node.child(index)
      {
        Self::validate_node(context, document, child, diagnostics);
      }
    }
  }
}
