use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
  pub attributes: Vec<Attribute>,
  pub body: Vec<TextNode>,
  pub content: String,
  pub dependencies: Vec<Dependency>,
  pub name: TextNode,
  pub parameters: Vec<Parameter>,
  pub range: lsp::Range,
  pub shebang: Option<TextNode>,
}

impl Recipe {
  pub(crate) fn body_lines(&self) -> impl Iterator<Item = RecipeLine<'_>> {
    self.body.iter().filter_map(RecipeLine::parse)
  }

  #[must_use]
  pub fn find_attribute(&self, name: &str) -> Option<&Attribute> {
    self
      .attributes
      .iter()
      .find(|attribute| attribute.name.value == name)
  }

  #[must_use]
  pub fn groups(&self) -> GroupSet {
    GroupSet::from_attributes(&self.attributes)
  }

  #[must_use]
  pub fn has_attribute(&self, name: &str) -> bool {
    self
      .attributes
      .iter()
      .any(|attribute| attribute.name.value == name)
  }

  #[must_use]
  pub fn runs_as_script(&self, default_script: bool) -> bool {
    if self.has_attribute("shell") {
      return false;
    }

    self.has_attribute("script") || self.shebang.is_some() || default_script
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn body_lines_comments() {
    let document = Document::from(indoc! {
      "
      foo:
        # bar
        baz
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0]
        .body_lines()
        .map(|line| (line.number, line.indent))
        .collect::<Vec<_>>(),
      [(1, "  "), (2, "  ")],
    );
  }

  #[test]
  fn body_lines_continuation() {
    let document = Document::from(indoc! {
      "
      foo:
        {{'bar'}}\\
      \t{{'baz'}}
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0]
        .body_lines()
        .map(|line| (line.number, line.indent, line.continues))
        .collect::<Vec<_>>(),
      [(1, "  ", true), (2, "\t", false)],
    );
  }

  #[test]
  fn body_lines_crlf() {
    let document = Document::from("[private]\r\nfoo:\r\n\tbar");

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0]
        .body_lines()
        .map(|line| (line.number, line.indent))
        .collect::<Vec<_>>(),
      [(2, "\t")],
    );
  }

  #[test]
  fn body_lines_empty() {
    let document = Document::from(indoc! {
      "
      foo:
      bar:
        baz
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert!(document.recipes()[0].body_lines().next().is_none());
  }

  #[test]
  fn body_lines_multiline_header() {
    let document = Document::from(indoc! {
      "
      foo: \\
      \tbar
        baz
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0]
        .body_lines()
        .map(|line| (line.number, line.indent))
        .collect::<Vec<_>>(),
      [(2, "  ")],
    );
  }

  #[test]
  fn body_lines_multiline_interpolation() {
    let document = Document::from(indoc! {
      "
      foo:
        {{'
      bar
          baz
      '}}
        qux
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0]
        .body_lines()
        .map(|line| (line.number, line.indent))
        .collect::<Vec<_>>(),
      [(1, "  "), (5, "  ")],
    );
  }

  #[test]
  fn body_lines_multiline_parameter() {
    let document = Document::from(indoc! {
      "
      foo bar='''
      \tbaz
          qux
      ''':
        bar
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0]
        .body_lines()
        .map(|line| (line.number, line.indent))
        .collect::<Vec<_>>(),
      [(4, "  ")],
    );
  }

  #[test]
  fn body_lines_recipe_boundary() {
    let document = Document::from(indoc! {
      "
      foo:
        bar
      baz:
      \tqux
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0]
        .body_lines()
        .map(|line| (line.number, line.indent))
        .collect::<Vec<_>>(),
      [(1, "  ")],
    );
  }

  #[test]
  fn body_lines_shebang() {
    let document = Document::from(indoc! {
      "
      foo:
        #!/bin/sh
        bar
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0]
        .body_lines()
        .map(|line| (line.number, line.indent))
        .collect::<Vec<_>>(),
      [(1, "  "), (2, "  ")],
    );
  }

  #[test]
  fn body_lines_shebang_comment() {
    let document = Document::from(indoc! {
      "
      foo:
        bar
          #!/bin/sh
        qux
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0]
        .body_lines()
        .map(|line| (line.number, line.indent))
        .collect::<Vec<_>>(),
      [(1, "  "), (2, "    "), (3, "  ")],
    );
  }

  #[test]
  fn body_lines_whitespace() {
    let document = Document::from(indoc! {
      "
      foo:
        bar

       \t
        baz
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0]
        .body_lines()
        .map(|line| (line.number, line.indent))
        .collect::<Vec<_>>(),
      [(1, "  "), (4, "  ")],
    );
  }

  #[test]
  fn recipe_groups_all_attributes() {
    let recipe = Recipe {
      name: TextNode {
        value: "test".into(),
        range: lsp::Range::at(8, 0, 8, 4),
      },
      attributes: vec![
        Attribute {
          name: TextNode {
            value: "linux".to_string(),
            range: lsp::Range::at(0, 1, 0, 6),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: lsp::Range::at(0, 0, 1, 0),
        },
        Attribute {
          name: TextNode {
            value: "windows".to_string(),
            range: lsp::Range::at(1, 1, 1, 8),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: lsp::Range::at(1, 0, 2, 0),
        },
        Attribute {
          name: TextNode {
            value: "macos".to_string(),
            range: lsp::Range::at(2, 1, 2, 6),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: lsp::Range::at(2, 0, 3, 0),
        },
        Attribute {
          name: TextNode {
            value: "unix".to_string(),
            range: lsp::Range::at(3, 1, 3, 5),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: lsp::Range::at(3, 0, 4, 0),
        },
        Attribute {
          name: TextNode {
            value: "dragonfly".to_string(),
            range: lsp::Range::at(4, 1, 4, 10),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: lsp::Range::at(4, 0, 5, 0),
        },
        Attribute {
          name: TextNode {
            value: "freebsd".to_string(),
            range: lsp::Range::at(5, 1, 5, 8),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: lsp::Range::at(5, 0, 6, 0),
        },
        Attribute {
          name: TextNode {
            value: "netbsd".to_string(),
            range: lsp::Range::at(6, 1, 6, 7),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: lsp::Range::at(6, 0, 7, 0),
        },
        Attribute {
          name: TextNode {
            value: "openbsd".to_string(),
            range: lsp::Range::at(7, 1, 7, 8),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: lsp::Range::at(7, 0, 8, 0),
        },
      ],
      dependencies: vec![],
      shebang: None,
      parameters: vec![],
      content:
        "[linux]\n[windows]\n[macos]\n[unix]\n[dragonfly]\n[freebsd]\n[netbsd]\n[openbsd]\ntest:\n  echo test"
          .to_string(),
      body: vec![TextNode {
        value: "  echo test".into(),
        range: lsp::Range::at(9, 0, 9, 11),
      }],
      range: lsp::Range::at(0, 0, 10, 0),
    };

    assert_eq!(
      recipe.groups(),
      GroupSet::from([
        Group::Android,
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
  fn recipe_groups_multiple_attributes() {
    let recipe = Recipe {
      name: TextNode {
        value: "test".into(),
        range: lsp::Range::at(2, 0, 2, 4),
      },
      attributes: vec![
        Attribute {
          name: TextNode {
            value: "linux".to_string(),
            range: lsp::Range::at(0, 1, 0, 6),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: lsp::Range::at(0, 0, 1, 0),
        },
        Attribute {
          name: TextNode {
            value: "windows".to_string(),
            range: lsp::Range::at(1, 1, 1, 8),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: lsp::Range::at(1, 0, 2, 0),
        },
      ],
      dependencies: vec![],
      shebang: None,
      parameters: vec![],
      content: "[linux]\n[windows]\ntest:\n  echo test".to_string(),
      body: vec![TextNode {
        value: "  echo test".into(),
        range: lsp::Range::at(3, 0, 3, 11),
      }],
      range: lsp::Range::at(0, 0, 4, 0),
    };

    assert_eq!(
      recipe.groups(),
      GroupSet::from([Group::Linux, Group::Windows])
    );
  }

  #[test]
  fn recipe_groups_no_attributes() {
    let recipe = Recipe {
      name: TextNode {
        value: "test".into(),
        range: lsp::Range::at(0, 0, 0, 4),
      },
      attributes: vec![],
      dependencies: vec![],
      shebang: None,
      parameters: vec![],
      content: "test:\n  echo test".to_string(),
      body: vec![TextNode {
        value: "  echo test".into(),
        range: lsp::Range::at(1, 0, 1, 11),
      }],
      range: lsp::Range::at(0, 0, 2, 0),
    };

    assert_eq!(recipe.groups(), GroupSet::from([Group::Any]));
  }

  #[test]
  fn recipe_groups_non_os_attributes() {
    let recipe = Recipe {
      name: TextNode {
        value: "test".into(),
        range: lsp::Range::at(1, 0, 1, 4),
      },
      attributes: vec![Attribute {
        name: TextNode {
          value: "private".to_string(),
          range: lsp::Range::at(0, 1, 0, 8),
        },
        arguments: vec![],
        target: Some(AttributeTarget::Recipe),
        range: lsp::Range::at(0, 0, 1, 0),
      }],
      dependencies: vec![],
      shebang: None,
      parameters: vec![],
      content: "[private]\ntest:\n  echo test".to_string(),
      body: vec![TextNode {
        value: "  echo test".into(),
        range: lsp::Range::at(2, 0, 2, 11),
      }],
      range: lsp::Range::at(0, 0, 3, 0),
    };

    assert_eq!(recipe.groups(), GroupSet::from([Group::Any]));
  }

  #[test]
  fn recipe_groups_single_attribute() {
    let recipe = Recipe {
      name: TextNode {
        value: "test".into(),
        range: lsp::Range::at(1, 0, 1, 4),
      },
      attributes: vec![Attribute {
        name: TextNode {
          value: "linux".to_string(),
          range: lsp::Range::at(0, 1, 0, 6),
        },
        arguments: vec![],
        target: Some(AttributeTarget::Recipe),
        range: lsp::Range::at(0, 0, 1, 0),
      }],
      dependencies: vec![],
      shebang: None,
      parameters: vec![],
      content: "[linux]\ntest:\n  echo test".to_string(),
      body: vec![TextNode {
        value: "  echo test".into(),
        range: lsp::Range::at(2, 0, 2, 11),
      }],
      range: lsp::Range::at(0, 0, 3, 0),
    };

    assert_eq!(recipe.groups(), GroupSet::from([Group::Linux]));
  }

  #[test]
  fn runs_as_script_with_default_script() {
    let recipe = Document::from("foo:\n  echo foo")
      .recipes()
      .into_iter()
      .next()
      .unwrap();

    assert!(!recipe.runs_as_script(false));
    assert!(recipe.runs_as_script(true));
  }

  #[test]
  fn runs_as_script_with_script_attribute() {
    let recipe = Document::from("[script]\nfoo:\n  echo foo")
      .recipes()
      .into_iter()
      .next()
      .unwrap();

    assert!(recipe.runs_as_script(false));
  }

  #[test]
  fn runs_as_script_with_shebang() {
    let recipe = Document::from("foo:\n  #!/usr/bin/env sh\n  echo foo")
      .recipes()
      .into_iter()
      .next()
      .unwrap();

    assert!(recipe.runs_as_script(false));
  }

  #[test]
  fn shell_attribute_overrides_default_script() {
    let recipe = Document::from("[shell]\nfoo:\n  echo foo")
      .recipes()
      .into_iter()
      .next()
      .unwrap();

    assert!(!recipe.runs_as_script(true));
  }
}
