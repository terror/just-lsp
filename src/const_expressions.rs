use super::*;

pub(crate) struct ConstExpressions<'a> {
  assignments: HashMap<String, Located<Variable>>,
  variables: HashMap<String, Value>,
  view: &'a ProjectView<'a>,
}

#[derive(Clone, PartialEq)]
enum Value {
  Known(Option<String>),
  NonConst,
  Unknown,
}

impl Value {
  fn boolean(value: bool) -> Self {
    Self::Known(value.then(|| "true".into()))
  }

  fn merge(self, other: &Self) -> Self {
    if self == *other { self } else { Self::Unknown }
  }

  fn truth(&self) -> Option<bool> {
    match self {
      Self::Known(value) => Some(value.is_some()),
      Self::NonConst | Self::Unknown => None,
    }
  }
}

impl<'a> ConstExpressions<'a> {
  fn conditional(&mut self, clauses: &[Node], document: &Document) -> Value {
    let Some((clause, rest)) = clauses.split_first() else {
      return Value::Known(None);
    };

    let Some(body) = clause.child_by_field_name("body") else {
      return Value::Unknown;
    };

    let Some(condition) = clause.find("^condition") else {
      return self.expression(body, document);
    };

    let condition = self.expression(condition, document);

    if condition == Value::NonConst {
      return condition;
    }

    match condition.truth() {
      Some(true) => self.expression(body, document),
      Some(false) => self.conditional(rest, document),
      None => {
        let value = self.expression(body, document);
        value.merge(&self.conditional(rest, document))
      }
    }
  }

  fn expression(&mut self, node: Node, document: &Document) -> Value {
    match node.kind() {
      "function_call" | "external_command" => return Value::NonConst,
      "identifier" => return self.variable(&document.get_node_text(&node)),
      "if_expression" => {
        let clauses = once(node)
          .chain(node.children(&mut node.walk()).filter(|child| {
            matches!(child.kind(), "else_if_clause" | "else_clause")
          }))
          .collect::<Vec<_>>();

        return self.conditional(&clauses, document);
      }
      "assert_expression" => {
        let Some(condition) = node.child_by_field_name("condition") else {
          return Value::Unknown;
        };

        let condition = self.expression(condition, document);

        if condition == Value::NonConst {
          return condition;
        }

        if condition.truth() == Some(false)
          && let Some(message) = node.child_by_field_name("message")
          && self.expression(message, document) == Value::NonConst
        {
          return Value::NonConst;
        }

        return Value::Unknown;
      }
      "string" if node.find("format_string").is_none() => {
        return StringLiteral::parse(&document.get_node_text(&node))
          .ok()
          .flatten()
          .filter(|literal| !literal.shell_expanded)
          .map_or(Value::Unknown, |literal| {
            Value::Known(Some(literal.cooked))
          });
      }
      _ => {}
    }

    let children = node
      .children(&mut node.walk())
      .filter(|child| child.kind() != "comment")
      .collect::<Vec<_>>();

    if node.kind() == "expression" {
      if let Some(operator) = children
        .iter()
        .position(|child| matches!(child.kind(), "&&" | "||"))
      {
        let left = self.sequence(&children[..operator], document);

        if left == Value::NonConst {
          return left;
        }

        return match left.truth() {
          Some(truth) if truth == (children[operator].kind() == "&&") => {
            self.sequence(&children[operator + 1..], document)
          }
          Some(_) => left,
          None => Value::Unknown,
        };
      }

      if let [left, operator, right] = children.as_slice()
        && matches!(operator.kind(), "==" | "!=")
      {
        let left = self.expression(*left, document);
        let right = self.expression(*right, document);

        return match (left, right) {
          (Value::NonConst, _) | (_, Value::NonConst) => Value::NonConst,
          (Value::Known(left), Value::Known(right)) => {
            Value::boolean((left == right) == (operator.kind() == "=="))
          }
          _ => Value::Unknown,
        };
      }

      return self.sequence(&children, document);
    }

    if matches!(node.kind(), "value" | "condition" | "regex_literal") {
      return node
        .named_child(0)
        .map_or(Value::Unknown, |child| self.expression(child, document));
    }

    if node.kind() == "not_expression" {
      let value = node
        .named_child(0)
        .map_or(Value::Unknown, |child| self.expression(child, document));

      return if value == Value::NonConst {
        value
      } else {
        value
          .truth()
          .map_or(Value::Unknown, |truth| Value::boolean(!truth))
      };
    }

    if node.kind() == "list_literal" && node.named_child_count() == 0 {
      return Value::Known(None);
    }

    if children
      .iter()
      .any(|child| self.expression(*child, document) == Value::NonConst)
    {
      Value::NonConst
    } else {
      Value::Unknown
    }
  }

  pub(crate) fn is_non_const(&mut self, expression: &TextNode) -> bool {
    Self::node(self.view.document(), expression.range).is_some_and(|node| {
      self.expression(node, self.view.document()) == Value::NonConst
    })
  }

  pub(crate) fn new(view: &'a ProjectView<'a>) -> Self {
    Self {
      assignments: view.resolved_variables(),
      variables: HashMap::new(),
      view,
    }
  }

  fn node(document: &Document, range: lsp::Range) -> Option<Node<'_>> {
    let byte = |position| {
      document
        .content
        .char_to_byte(document.content.lsp_position_to_char(position))
    };

    document
      .tree
      .root_node()
      .descendant_for_byte_range(byte(range.start), byte(range.end))
  }

  fn sequence(&mut self, nodes: &[Node], document: &Document) -> Value {
    if let [node] = nodes {
      return self.expression(*node, document);
    }

    if nodes
      .iter()
      .any(|node| self.expression(*node, document) == Value::NonConst)
    {
      Value::NonConst
    } else {
      Value::Unknown
    }
  }

  fn variable(&mut self, name: &str) -> Value {
    if let Some(value) = self.variables.get(name) {
      return value.clone();
    }

    self.variables.insert(name.into(), Value::Unknown);

    let expression = self.assignments.get(name).and_then(|variable| {
      if variable
        .attributes
        .iter()
        .any(|attribute| attribute.condition().is_some())
      {
        return None;
      }

      let document = self
        .view
        .documents()
        .find(|document| document.uri == variable.uri)?;

      let node = Self::node(document, variable.range)?;

      let value = node.child_by_field_name("right")?;

      Some((value, document))
    });

    let value = expression.map_or(Value::Unknown, |(value, document)| {
      self.expression(value, document)
    });

    self.variables.insert(name.into(), value.clone());

    value
  }
}
