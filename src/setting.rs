use super::*;

#[derive(Debug)]
pub enum SettingKind {
  Array,
  Boolean(bool),
  String,
}

impl Display for SettingKind {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      SettingKind::Array => write!(f, "array"),
      SettingKind::Boolean(_) => write!(f, "boolean"),
      SettingKind::String => write!(f, "string"),
    }
  }
}

impl PartialEq for SettingKind {
  fn eq(&self, other: &Self) -> bool {
    matches!(
      (self, other),
      (SettingKind::Array, SettingKind::Array)
        | (SettingKind::Boolean(_), SettingKind::Boolean(_))
        | (SettingKind::String, SettingKind::String)
    )
  }
}

#[derive(Debug, PartialEq)]
pub struct Setting {
  pub kind: SettingKind,
  pub name: TextNode,
  pub range: lsp::Range,
}

impl Setting {
  #[must_use]
  pub fn from_node(node: &Node, document: &Document) -> Option<Self> {
    let range = node.get_range(document);

    let name_node = node.child(1)?;

    let name = TextNode {
      value: document.get_node_text(&name_node),
      range: name_node.get_range(document),
    };

    let mut cursor = node.walk();

    let right_children = node
      .children_by_field_name("right", &mut cursor)
      .collect::<Vec<_>>();

    let has_bracket = right_children.iter().any(|child| child.kind() == "[");

    let boolean_child = right_children
      .iter()
      .find(|child| child.kind() == "boolean");

    let string_child =
      right_children.iter().find(|child| child.kind() == "string");

    let kind = if has_bracket {
      SettingKind::Array
    } else if let Some(boolean) = boolean_child {
      SettingKind::Boolean(document.get_node_text(boolean) == "true")
    } else if string_child.is_some() {
      SettingKind::String
    } else if right_children.is_empty() {
      SettingKind::Boolean(true)
    } else {
      return None;
    };

    Some(Setting { kind, name, range })
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
  fn parse_string() {
    assert_eq!(
      Document::from("set foo := \"bar\"\n").settings(),
      vec![Setting {
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
          name: TextNode {
            value: "foo".into(),
            range: lsp::Range::at(0, 4, 0, 7),
          },
          kind: SettingKind::Boolean(true),
          range: lsp::Range::at(0, 0, 1, 0),
        },
        Setting {
          name: TextNode {
            value: "bar".into(),
            range: lsp::Range::at(1, 4, 1, 7),
          },
          kind: SettingKind::String,
          range: lsp::Range::at(1, 0, 2, 0),
        },
        Setting {
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
}
