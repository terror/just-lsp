use super::*;

#[derive(Debug, PartialEq)]
pub struct Setting {
  pub attributes: Vec<Attribute>,
  pub kind: SettingKind,
  pub name: TextNode,
  pub range: lsp::Range,
}

impl Setting {
  pub fn has_attribute(&self, name: &str) -> bool {
    self
      .attributes
      .iter()
      .any(|attribute| attribute.name.value == name)
  }

  #[must_use]
  pub fn from_node(node: &Node, document: &Document) -> Option<Self> {
    let range = node.get_range(document);

    let name_node = node.child_by_field_name("left")?;

    let name = TextNode {
      value: document.get_node_text(&name_node),
      range: name_node.get_range(document),
    };

    let mut cursor = node.walk();

    let right_children = node
      .children_by_field_name("right", &mut cursor)
      .collect::<Vec<_>>();

    let boolean_child = right_children
      .iter()
      .find(|child| child.kind() == "boolean");

    let string_child =
      right_children.iter().find(|child| child.kind() == "string");

    let expression_child = right_children
      .iter()
      .find(|child| child.kind() == "expression");

    let kind = if node.find("list_literal").is_some() {
      SettingKind::Array
    } else if let Some(boolean) = boolean_child {
      SettingKind::Boolean(document.get_node_text(boolean) == "true")
    } else if string_child.is_some() || expression_child.is_some() {
      SettingKind::String
    } else if right_children.is_empty() {
      SettingKind::Boolean(true)
    } else {
      return None;
    };

    let attributes = node
      .find_all("attribute")
      .into_iter()
      .flat_map(|attribute_node| {
        attribute_node
          .find_all("^identifier")
          .into_iter()
          .map(|identifier_node| {
            let arguments = identifier_node
              .siblings()
              .take_while(|sibling| sibling.kind() != "identifier")
              .filter(|sibling| {
                sibling.start_byte() != sibling.end_byte()
                  && matches!(
                    sibling.kind(),
                    "string" | "expression" | "attribute_named_param"
                  )
              })
              .map(|argument_node| TextNode {
                value: document.get_node_text(&argument_node),
                range: argument_node.get_range(document),
              })
              .collect::<Vec<_>>();

            Attribute {
              name: TextNode {
                value: document.get_node_text(&identifier_node),
                range: identifier_node.get_range(document),
              },
              arguments,
              target: Some(AttributeTarget::Setting),
              range: attribute_node.get_range(document),
            }
          })
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();

    Some(Setting {
      attributes,
      kind,
      name,
      range,
    })
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  #[test]
  fn parse_boolean_with_value() {
    assert_eq!(
      Document::from("set foo := true\n").settings(),
      vec![Setting {
        attributes: vec![],
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(0, 4, 0, 7),
        },
        kind: SettingKind::Boolean(true),
        range: lsp::Range::at(0, 0, 1, 0),
      }],
    );
  }

  #[test]
  fn parse_boolean_false() {
    assert_eq!(
      Document::from("set foo := false\n").settings(),
      vec![Setting {
        attributes: vec![],
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(0, 4, 0, 7),
        },
        kind: SettingKind::Boolean(false),
        range: lsp::Range::at(0, 0, 1, 0),
      }],
    );
  }

  #[test]
  fn parse_boolean_without_value() {
    assert_eq!(
      Document::from("set export\n").settings(),
      vec![Setting {
        attributes: vec![],
        name: TextNode {
          value: "export".into(),
          range: lsp::Range::at(0, 4, 0, 10),
        },
        kind: SettingKind::Boolean(true),
        range: lsp::Range::at(0, 0, 1, 0),
      }],
    );
  }

  #[test]
  fn parse_array() {
    assert_eq!(
      Document::from("set shell := [\"zsh\", \"-cu\"]\n").settings(),
      vec![Setting {
        attributes: vec![],
        name: TextNode {
          value: "shell".into(),
          range: lsp::Range::at(0, 4, 0, 9),
        },
        kind: SettingKind::Array,
        range: lsp::Range::at(0, 0, 1, 0),
      }],
    );
  }

  #[test]
  fn parse_hyphenated_array() {
    assert_eq!(
      Document::from(
        "set windows-shell := [\"powershell.exe\", \"-NoLogo\", \"-Command\"]\n"
      )
      .settings(),
      vec![Setting {
        attributes: vec![],
        name: TextNode {
          value: "windows-shell".into(),
          range: lsp::Range::at(0, 4, 0, 17),
        },
        kind: SettingKind::Array,
        range: lsp::Range::at(0, 0, 1, 0),
      }],
    );
  }

  #[test]
  fn parse_string() {
    assert_eq!(
      Document::from("set foo := \"bar\"\n").settings(),
      vec![Setting {
        attributes: vec![],
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(0, 4, 0, 7),
        },
        kind: SettingKind::String,
        range: lsp::Range::at(0, 0, 1, 0),
      }],
    );
  }

  #[test]
  fn parse_string_containing_walrus() {
    assert_eq!(
      Document::from("set foo := \"bar := baz\"\n").settings(),
      vec![Setting {
        attributes: vec![],
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(0, 4, 0, 7),
        },
        kind: SettingKind::String,
        range: lsp::Range::at(0, 0, 1, 0),
      }],
    );
  }

  #[test]
  fn parse_expression() {
    assert_eq!(
      Document::from("set foo := \"bar\" / baz\n").settings(),
      vec![Setting {
        attributes: vec![],
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(0, 4, 0, 7),
        },
        kind: SettingKind::String,
        range: lsp::Range::at(0, 0, 1, 0),
      }],
    );
  }

  #[test]
  fn parse_multiple_settings() {
    assert_eq!(
      Document::from(indoc! {"
        set foo := true
        set bar := \"baz\"
        set shell := [\"zsh\", \"-cu\"]
      "})
      .settings(),
      vec![
        Setting {
          attributes: vec![],
          name: TextNode {
            value: "foo".into(),
            range: lsp::Range::at(0, 4, 0, 7),
          },
          kind: SettingKind::Boolean(true),
          range: lsp::Range::at(0, 0, 1, 0),
        },
        Setting {
          attributes: vec![],
          name: TextNode {
            value: "bar".into(),
            range: lsp::Range::at(1, 4, 1, 7),
          },
          kind: SettingKind::String,
          range: lsp::Range::at(1, 0, 2, 0),
        },
        Setting {
          attributes: vec![],
          name: TextNode {
            value: "shell".into(),
            range: lsp::Range::at(2, 4, 2, 9),
          },
          kind: SettingKind::Array,
          range: lsp::Range::at(2, 0, 3, 0),
        },
      ],
    );
  }

  #[test]
  fn parse_attributes() {
    assert_eq!(
      Document::from("[group(\"foo\")]\nset bar := true\n").settings(),
      vec![Setting {
        attributes: vec![Attribute {
          name: TextNode {
            value: "group".into(),
            range: lsp::Range::at(0, 1, 0, 6),
          },
          arguments: vec![TextNode {
            value: "\"foo\"".into(),
            range: lsp::Range::at(0, 7, 0, 12),
          }],
          target: Some(AttributeTarget::Setting),
          range: lsp::Range::at(0, 0, 1, 0),
        }],
        name: TextNode {
          value: "bar".into(),
          range: lsp::Range::at(1, 4, 1, 7),
        },
        kind: SettingKind::Boolean(true),
        range: lsp::Range::at(0, 0, 2, 0),
      }],
    );
  }
}
