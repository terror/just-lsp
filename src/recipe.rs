use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Dependency {
  pub(crate) name: String,
  pub(crate) arguments: Vec<TextNode>,
  pub(crate) range: lsp::Range,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ParameterKind {
  Normal,
  Variadic(VariadicType),
  Export,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum VariadicType {
  OneOrMore,
  ZeroOrMore,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Parameter {
  pub(crate) name: String,
  pub(crate) kind: ParameterKind,
  pub(crate) default_value: Option<String>,
  pub(crate) content: String,
  pub(crate) range: lsp::Range,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct ParameterJson {
  pub(crate) name: String,
  pub(crate) default_value: Option<String>,
}

impl From<Parameter> for ParameterJson {
  fn from(param: Parameter) -> Self {
    ParameterJson {
      name: param.name,
      default_value: param.default_value,
    }
  }
}

impl Parameter {
  pub(crate) fn parse(text: &str, range: lsp::Range) -> Option<Self> {
    let parts: Vec<&str> = text.split('=').collect();

    let (param_name, default_value) = if parts.len() > 1 {
      (
        parts[0].trim(),
        Some(parts[1..].join("=").trim().to_string()),
      )
    } else {
      (text.trim(), None)
    };

    if param_name.is_empty() {
      return None;
    }

    let (name, kind) = if let Some(stripped) = param_name.strip_prefix('$') {
      (stripped.to_string(), ParameterKind::Export)
    } else if let Some(stripped) = param_name.strip_prefix('+') {
      (
        stripped.to_string(),
        ParameterKind::Variadic(VariadicType::OneOrMore),
      )
    } else if let Some(stripped) = param_name.strip_prefix('*') {
      (
        stripped.to_string(),
        ParameterKind::Variadic(VariadicType::ZeroOrMore),
      )
    } else {
      (param_name.to_string(), ParameterKind::Normal)
    };

    if name.is_empty() {
      return None;
    }

    Some(Parameter {
      name: name.trim().into(),
      kind,
      default_value,
      content: text.trim().to_string(),
      range,
    })
  }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Recipe {
  pub(crate) name: String,
  pub(crate) attributes: Vec<Attribute>,
  pub(crate) dependencies: Vec<Dependency>,
  pub(crate) parameters: Vec<Parameter>,
  pub(crate) content: String,
  pub(crate) range: lsp::Range,
}

impl Recipe {
  pub(crate) fn os_groups(&self) -> HashSet<OsGroup> {
    let mut os_groups = HashSet::new();

    for attribute in &self.attributes {
      let attribute_name = attribute.name.value.as_str();

      if let Some(targets) = OsGroup::targets(attribute_name) {
        os_groups.extend(targets);
      }
    }

    if os_groups.is_empty() {
      os_groups.insert(OsGroup::Any);
    }

    os_groups
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn create_range(
    start_line: u32,
    start_char: u32,
    end_line: u32,
    end_char: u32,
  ) -> lsp::Range {
    lsp::Range {
      start: lsp::Position {
        line: start_line,
        character: start_char,
      },
      end: lsp::Position {
        line: end_line,
        character: end_char,
      },
    }
  }

  #[test]
  fn parse_normal_parameter() {
    let input = "target";

    let range = create_range(0, 0, 0, 6);

    let param = Parameter::parse(input, range).unwrap();

    assert_eq!(
      param,
      Parameter {
        name: "target".to_string(),
        kind: ParameterKind::Normal,
        default_value: None,
        content: "target".to_string(),
        range,
      }
    );
  }

  #[test]
  fn parse_parameter_with_default() {
    let input = "tests=\"default\"";

    let range = create_range(0, 0, 0, 15);

    let param = Parameter::parse(input, range).unwrap();

    assert_eq!(
      param,
      Parameter {
        name: "tests".to_string(),
        kind: ParameterKind::Normal,
        default_value: Some("\"default\"".to_string()),
        content: "tests=\"default\"".to_string(),
        range,
      }
    );
  }

  #[test]
  fn parse_parameter_with_complex_default() {
    let input = "triple=(arch + \"-unknown-unknown\")";

    let range = create_range(0, 0, 0, 32);

    let param = Parameter::parse(input, range).unwrap();

    assert_eq!(
      param,
      Parameter {
        name: "triple".to_string(),
        kind: ParameterKind::Normal,
        default_value: Some("(arch + \"-unknown-unknown\")".to_string()),
        content: "triple=(arch + \"-unknown-unknown\")".to_string(),
        range,
      }
    );
  }

  #[test]
  fn parse_export_parameter() {
    let input = "$bar";

    let range = create_range(0, 0, 0, 4);

    let param = Parameter::parse(input, range).unwrap();

    assert_eq!(
      param,
      Parameter {
        name: "bar".to_string(),
        kind: ParameterKind::Export,
        default_value: None,
        content: "$bar".to_string(),
        range,
      }
    );
  }

  #[test]
  fn parse_variadic_one_or_more_parameter() {
    let input = "+FILES";

    let range = create_range(0, 0, 0, 6);

    let param = Parameter::parse(input, range).unwrap();

    assert_eq!(
      param,
      Parameter {
        name: "FILES".to_string(),
        kind: ParameterKind::Variadic(VariadicType::OneOrMore),
        default_value: None,
        content: "+FILES".to_string(),
        range,
      }
    );
  }

  #[test]
  fn parse_variadic_zero_or_more_parameter() {
    let input = "*FLAGS";

    let range = create_range(0, 0, 0, 6);

    let param = Parameter::parse(input, range).unwrap();

    assert_eq!(
      param,
      Parameter {
        name: "FLAGS".to_string(),
        kind: ParameterKind::Variadic(VariadicType::ZeroOrMore),
        default_value: None,
        content: "*FLAGS".to_string(),
        range,
      }
    );
  }

  #[test]
  fn parse_variadic_with_default() {
    let input = "+FLAGS='-q'";

    let range = create_range(0, 0, 0, 12);

    let param = Parameter::parse(input, range).unwrap();

    assert_eq!(
      param,
      Parameter {
        name: "FLAGS".to_string(),
        kind: ParameterKind::Variadic(VariadicType::OneOrMore),
        default_value: Some("'-q'".to_string()),
        content: "+FLAGS='-q'".to_string(),
        range,
      }
    );
  }

  #[test]
  fn invalid_parameter_input() {
    let range = create_range(0, 0, 0, 0);

    assert_eq!(Parameter::parse("", range), None);
    assert_eq!(Parameter::parse("$", range), None);
    assert_eq!(Parameter::parse("+", range), None);
    assert_eq!(Parameter::parse("*", range), None);
  }

  #[test]
  fn recipe_os_groups_no_attributes() {
    let recipe = Recipe {
      name: "test".to_string(),
      attributes: vec![],
      dependencies: vec![],
      parameters: vec![],
      content: "test:\n  echo test".to_string(),
      range: create_range(0, 0, 2, 0),
    };

    assert_eq!(recipe.os_groups(), HashSet::from([OsGroup::Any]));
  }

  #[test]
  fn recipe_os_groups_single_attribute() {
    let recipe = Recipe {
      name: "test".to_string(),
      attributes: vec![Attribute {
        name: TextNode {
          value: "linux".to_string(),
          range: create_range(0, 1, 0, 6),
        },
        arguments: vec![],
        range: create_range(0, 0, 1, 0),
      }],
      dependencies: vec![],
      parameters: vec![],
      content: "[linux]\ntest:\n  echo test".to_string(),
      range: create_range(0, 0, 3, 0),
    };

    assert_eq!(recipe.os_groups(), HashSet::from([OsGroup::Linux]));
  }

  #[test]
  fn recipe_os_groups_multiple_attributes() {
    let recipe = Recipe {
      name: "test".to_string(),
      attributes: vec![
        Attribute {
          name: TextNode {
            value: "linux".to_string(),
            range: create_range(0, 1, 0, 6),
          },
          arguments: vec![],
          range: create_range(0, 0, 1, 0),
        },
        Attribute {
          name: TextNode {
            value: "windows".to_string(),
            range: create_range(1, 1, 1, 8),
          },
          arguments: vec![],
          range: create_range(1, 0, 2, 0),
        },
      ],
      dependencies: vec![],
      parameters: vec![],
      content: "[linux]\n[windows]\ntest:\n  echo test".to_string(),
      range: create_range(0, 0, 4, 0),
    };

    assert_eq!(
      recipe.os_groups(),
      HashSet::from([OsGroup::Linux, OsGroup::Windows])
    );
  }

  #[test]
  fn recipe_os_groups_all_attributes() {
    let recipe = Recipe {
      name: "test".to_string(),
      attributes: vec![
        Attribute {
          name: TextNode {
            value: "linux".to_string(),
            range: create_range(0, 1, 0, 6),
          },
          arguments: vec![],
          range: create_range(0, 0, 1, 0),
        },
        Attribute {
          name: TextNode {
            value: "windows".to_string(),
            range: create_range(1, 1, 1, 8),
          },
          arguments: vec![],
          range: create_range(1, 0, 2, 0),
        },
        Attribute {
          name: TextNode {
            value: "macos".to_string(),
            range: create_range(2, 1, 2, 6),
          },
          arguments: vec![],
          range: create_range(2, 0, 3, 0),
        },
        Attribute {
          name: TextNode {
            value: "unix".to_string(),
            range: create_range(3, 1, 3, 5),
          },
          arguments: vec![],
          range: create_range(3, 0, 4, 0),
        },
        Attribute {
          name: TextNode {
            value: "openbsd".to_string(),
            range: create_range(4, 1, 4, 8),
          },
          arguments: vec![],
          range: create_range(4, 0, 5, 0),
        },
      ],
      dependencies: vec![],
      parameters: vec![],
      content:
        "[linux]\n[windows]\n[macos]\n[unix]\n[openbsd]\ntest:\n  echo test"
          .to_string(),
      range: create_range(0, 0, 7, 0),
    };

    assert_eq!(
      recipe.os_groups(),
      HashSet::from([
        OsGroup::Linux,
        OsGroup::Windows,
        OsGroup::Macos,
        OsGroup::Openbsd
      ])
    );
  }

  #[test]
  fn recipe_os_groups_non_os_attributes() {
    let recipe = Recipe {
      name: "test".to_string(),
      attributes: vec![Attribute {
        name: TextNode {
          value: "private".to_string(),
          range: create_range(0, 1, 0, 8),
        },
        arguments: vec![],
        range: create_range(0, 0, 1, 0),
      }],
      dependencies: vec![],
      parameters: vec![],
      content: "[private]\ntest:\n  echo test".to_string(),
      range: create_range(0, 0, 3, 0),
    };

    assert_eq!(recipe.os_groups(), HashSet::from([OsGroup::Any]));
  }
}
