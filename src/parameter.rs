use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterKind {
  Export,
  Normal,
  Variadic(VariadicType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariadicType {
  OneOrMore,
  ZeroOrMore,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
  pub content: String,
  pub default_value: Option<String>,
  pub kind: ParameterKind,
  pub name: String,
  pub range: lsp::Range,
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
      let kind = if node.child(0).is_some_and(|child| child.kind() == "$") {
        ParameterKind::Export
      } else {
        ParameterKind::Normal
      };

      (*node, kind)
    };

    let name =
      document.get_node_text(&parameter_node.child_by_field_name("name")?);

    let default_value = parameter_node
      .child_by_field_name("default")
      .map(|node| document.get_node_text(&node));

    Some(Parameter {
      name,
      kind,
      default_value,
      content: document.get_node_text(node).trim().to_string(),
      range: node.get_range(document),
    })
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn parse_normal_parameter() {
    assert_eq!(
      Document::from("foo target:\n").recipes()[0].parameters,
      vec![Parameter {
        name: "target".to_string(),
        kind: ParameterKind::Normal,
        default_value: None,
        content: "target".to_string(),
        range: lsp::Range::at(0, 4, 0, 10),
      }],
    );
  }

  #[test]
  fn parse_parameter_with_default() {
    assert_eq!(
      Document::from("foo tests=\"default\":\n").recipes()[0].parameters,
      vec![Parameter {
        name: "tests".to_string(),
        kind: ParameterKind::Normal,
        default_value: Some("\"default\"".to_string()),
        content: "tests=\"default\"".to_string(),
        range: lsp::Range::at(0, 4, 0, 19),
      }],
    );
  }

  #[test]
  fn parse_parameter_with_complex_default() {
    assert_eq!(
      Document::from("foo triple=(arch + \"-unknown-unknown\"):\n").recipes()
        [0]
        .parameters,
      vec![Parameter {
        name: "triple".to_string(),
        kind: ParameterKind::Normal,
        default_value: Some("(arch + \"-unknown-unknown\")".to_string()),
        content: "triple=(arch + \"-unknown-unknown\")".to_string(),
        range: lsp::Range::at(0, 4, 0, 38),
      }],
    );
  }

  #[test]
  fn parse_export_parameter() {
    assert_eq!(
      Document::from("foo $bar:\n").recipes()[0].parameters,
      vec![Parameter {
        name: "bar".to_string(),
        kind: ParameterKind::Export,
        default_value: None,
        content: "$bar".to_string(),
        range: lsp::Range::at(0, 4, 0, 8),
      }],
    );
  }

  #[test]
  fn parse_variadic_one_or_more_parameter() {
    assert_eq!(
      Document::from("foo +FILES:\n").recipes()[0].parameters,
      vec![Parameter {
        name: "FILES".to_string(),
        kind: ParameterKind::Variadic(VariadicType::OneOrMore),
        default_value: None,
        content: "+FILES".to_string(),
        range: lsp::Range::at(0, 4, 0, 10),
      }],
    );
  }

  #[test]
  fn parse_variadic_zero_or_more_parameter() {
    assert_eq!(
      Document::from("foo *FLAGS:\n").recipes()[0].parameters,
      vec![Parameter {
        name: "FLAGS".to_string(),
        kind: ParameterKind::Variadic(VariadicType::ZeroOrMore),
        default_value: None,
        content: "*FLAGS".to_string(),
        range: lsp::Range::at(0, 4, 0, 10),
      }],
    );
  }

  #[test]
  fn parse_variadic_with_default() {
    assert_eq!(
      Document::from("foo +FLAGS='-q':\n").recipes()[0].parameters,
      vec![Parameter {
        name: "FLAGS".to_string(),
        kind: ParameterKind::Variadic(VariadicType::OneOrMore),
        default_value: Some("'-q'".to_string()),
        content: "+FLAGS='-q'".to_string(),
        range: lsp::Range::at(0, 4, 0, 15),
      }],
    );
  }

  #[test]
  fn invalid_parameter_input() {
    assert_eq!(Document::from("foo:\n").recipes()[0].parameters, vec![]);
    assert_eq!(Document::from("foo $:\n").recipes()[0].parameters, vec![]);
    assert_eq!(Document::from("foo +:\n").recipes()[0].parameters, vec![]);
    assert_eq!(Document::from("foo *:\n").recipes()[0].parameters, vec![]);
  }
}
