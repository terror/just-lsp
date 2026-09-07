use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterKind {
  Normal,
  Variadic(VariadicType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariadicType {
  OneOrMore,
  ZeroOrMore,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ParameterJson {
  pub default_value: Option<String>,
  pub name: String,
}

impl From<Parameter> for ParameterJson {
  fn from(parameter: Parameter) -> Self {
    ParameterJson {
      name: parameter.name,
      default_value: parameter.default_value,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
  pub content: String,
  pub default_value: Option<String>,
  pub export: bool,
  pub kind: ParameterKind,
  pub name: String,
  pub range: lsp::Range,
}

impl Parameter {
  #[must_use]
  pub fn from_node(node: &Node, document: &Document) -> Option<Self> {
    let (parameter_node, kind) = if node.kind() == "variadic_parameter" {
      let kleene = document.get_node_text(&node.child_by_field_name("kleene")?);

      let kind = match kleene.as_str() {
        "+" => ParameterKind::Variadic(VariadicType::OneOrMore),
        "*" => ParameterKind::Variadic(VariadicType::ZeroOrMore),
        _ => return None,
      };

      (node.find("^parameter")?, kind)
    } else {
      (*node, ParameterKind::Normal)
    };

    let export = parameter_node
      .child(0)
      .is_some_and(|child| child.kind() == "$");

    let name =
      document.get_node_text(&parameter_node.child_by_field_name("name")?);

    let default_value = parameter_node
      .child_by_field_name("default")
      .map(|node| document.get_node_text(&node));

    Some(Parameter {
      name,
      kind,
      export,
      default_value,
      content: document.get_node_text(node).trim().to_string(),
      range: node.get_range(document),
    })
  }
}
