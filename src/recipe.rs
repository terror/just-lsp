use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
  pub attributes: Vec<Attribute>,
  pub content: String,
  pub dependencies: Vec<Dependency>,
  pub name: TextNode,
  pub parameters: Vec<Parameter>,
  pub range: lsp::Range,
  pub shebang: Option<TextNode>,
}

impl Recipe {
  #[must_use]
  pub fn find_attribute(&self, name: &str) -> Option<&Attribute> {
    self
      .attributes
      .iter()
      .find(|attribute| attribute.name.value == name)
  }

  #[must_use]
  pub fn groups(&self) -> HashSet<Group> {
    let mut groups = HashSet::new();

    for attribute in &self.attributes {
      let attribute_name = attribute.name.value.as_str();

      if let Some(targets) = Group::targets(attribute_name) {
        groups.extend(targets);
      }
    }

    if groups.is_empty() {
      groups.insert(Group::Any);
    }

    groups
  }

  #[must_use]
  pub fn has_attribute(&self, name: &str) -> bool {
    self
      .attributes
      .iter()
      .any(|attribute| attribute.name.value == name)
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
  fn recipe_groups_no_attributes() {
    let recipe = Recipe {
      name: TextNode {
        value: "test".into(),
        range: create_range(0, 0, 0, 4),
      },
      attributes: vec![],
      dependencies: vec![],
      shebang: None,
      parameters: vec![],
      content: "test:\n  echo test".to_string(),
      range: create_range(0, 0, 2, 0),
    };

    assert_eq!(recipe.groups(), HashSet::from([Group::Any]));
  }

  #[test]
  fn recipe_groups_single_attribute() {
    let recipe = Recipe {
      name: TextNode {
        value: "test".into(),
        range: create_range(1, 0, 1, 4),
      },
      attributes: vec![Attribute {
        name: TextNode {
          value: "linux".to_string(),
          range: create_range(0, 1, 0, 6),
        },
        arguments: vec![],
        target: Some(AttributeTarget::Recipe),
        range: create_range(0, 0, 1, 0),
      }],
      dependencies: vec![],
      shebang: None,
      parameters: vec![],
      content: "[linux]\ntest:\n  echo test".to_string(),
      range: create_range(0, 0, 3, 0),
    };

    assert_eq!(recipe.groups(), HashSet::from([Group::Linux]));
  }

  #[test]
  fn recipe_groups_multiple_attributes() {
    let recipe = Recipe {
      name: TextNode {
        value: "test".into(),
        range: create_range(2, 0, 2, 4),
      },
      attributes: vec![
        Attribute {
          name: TextNode {
            value: "linux".to_string(),
            range: create_range(0, 1, 0, 6),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: create_range(0, 0, 1, 0),
        },
        Attribute {
          name: TextNode {
            value: "windows".to_string(),
            range: create_range(1, 1, 1, 8),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: create_range(1, 0, 2, 0),
        },
      ],
      dependencies: vec![],
      shebang: None,
      parameters: vec![],
      content: "[linux]\n[windows]\ntest:\n  echo test".to_string(),
      range: create_range(0, 0, 4, 0),
    };

    assert_eq!(
      recipe.groups(),
      HashSet::from([Group::Linux, Group::Windows])
    );
  }

  #[test]
  fn recipe_groups_all_attributes() {
    let recipe = Recipe {
      name: TextNode {
        value: "test".into(),
        range: create_range(8, 0, 8, 4),
      },
      attributes: vec![
        Attribute {
          name: TextNode {
            value: "linux".to_string(),
            range: create_range(0, 1, 0, 6),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: create_range(0, 0, 1, 0),
        },
        Attribute {
          name: TextNode {
            value: "windows".to_string(),
            range: create_range(1, 1, 1, 8),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: create_range(1, 0, 2, 0),
        },
        Attribute {
          name: TextNode {
            value: "macos".to_string(),
            range: create_range(2, 1, 2, 6),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: create_range(2, 0, 3, 0),
        },
        Attribute {
          name: TextNode {
            value: "unix".to_string(),
            range: create_range(3, 1, 3, 5),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: create_range(3, 0, 4, 0),
        },
        Attribute {
          name: TextNode {
            value: "dragonfly".to_string(),
            range: create_range(4, 1, 4, 10),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: create_range(4, 0, 5, 0),
        },
        Attribute {
          name: TextNode {
            value: "freebsd".to_string(),
            range: create_range(5, 1, 5, 8),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: create_range(5, 0, 6, 0),
        },
        Attribute {
          name: TextNode {
            value: "netbsd".to_string(),
            range: create_range(6, 1, 6, 7),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: create_range(6, 0, 7, 0),
        },
        Attribute {
          name: TextNode {
            value: "openbsd".to_string(),
            range: create_range(7, 1, 7, 8),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: create_range(7, 0, 8, 0),
        },
      ],
      dependencies: vec![],
      shebang: None,
      parameters: vec![],
      content:
        "[linux]\n[windows]\n[macos]\n[unix]\n[dragonfly]\n[freebsd]\n[netbsd]\n[openbsd]\ntest:\n  echo test"
          .to_string(),
      range: create_range(0, 0, 10, 0),
    };

    assert_eq!(
      recipe.groups(),
      HashSet::from([
        Group::Dragonfly,
        Group::Freebsd,
        Group::Linux,
        Group::Macos,
        Group::Netbsd,
        Group::Openbsd,
        Group::Windows,
      ])
    );
  }

  #[test]
  fn recipe_groups_non_os_attributes() {
    let recipe = Recipe {
      name: TextNode {
        value: "test".into(),
        range: create_range(1, 0, 1, 4),
      },
      attributes: vec![Attribute {
        name: TextNode {
          value: "private".to_string(),
          range: create_range(0, 1, 0, 8),
        },
        arguments: vec![],
        target: Some(AttributeTarget::Recipe),
        range: create_range(0, 0, 1, 0),
      }],
      dependencies: vec![],
      shebang: None,
      parameters: vec![],
      content: "[private]\ntest:\n  echo test".to_string(),
      range: create_range(0, 0, 3, 0),
    };

    assert_eq!(recipe.groups(), HashSet::from([Group::Any]));
  }
}
