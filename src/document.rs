use super::*;

#[derive(Debug)]
pub struct Document {
  pub(crate) content: Rope,
  pub(crate) tree: Option<Tree>,
  pub(crate) uri: lsp::Url,
  pub(crate) version: i32,
}

impl TryFrom<lsp::DidOpenTextDocumentParams> for Document {
  type Error = Box<dyn std::error::Error>;

  fn try_from(params: lsp::DidOpenTextDocumentParams) -> Result<Self> {
    let lsp::TextDocumentItem {
      text, uri, version, ..
    } = params.text_document;

    let mut document = Self {
      content: Rope::from_str(&text),
      tree: None,
      uri,
      version,
    };

    document.parse_incremental()?;

    Ok(document)
  }
}

impl Document {
  pub(crate) fn apply_change(
    &mut self,
    params: lsp::DidChangeTextDocumentParams,
  ) -> Result {
    let lsp::DidChangeTextDocumentParams {
      content_changes,
      text_document: lsp::VersionedTextDocumentIdentifier { version, .. },
      ..
    } = params;

    // Update version
    self.version = version;

    // Process each change incrementally
    for change in content_changes {
      let edit = self.content.build_edit(&change);

      // Apply edit to the Rope
      self.content.apply_edit(&edit);

      // Apply the same edit to the Tree-sitter tree
      if let Some(tree) = &mut self.tree {
        tree.edit(&edit.input_edit);
      }
    }

    // Reparse with the current tree as a starting point
    self.parse_incremental()?;

    Ok(())
  }

  pub(crate) fn find_recipe(&self, name: &str) -> Option<Recipe> {
    self
      .get_recipes()
      .into_iter()
      .find(|recipe| recipe.name == name)
  }

  pub(crate) fn get_aliases(&self) -> Vec<Alias> {
    let root = match &self.tree {
      Some(tree) => tree.root_node(),
      None => return Vec::new(),
    };

    root
      .find_all("alias")
      .iter()
      .filter_map(|alias_node| {
        let left_node = alias_node.find("^identifier[0]")?;

        let right_node = alias_node.find("^identifier[1]")?;

        Some(Alias {
          name: TextNode {
            value: self.get_node_text(&left_node),
            range: left_node.get_range(),
          },
          value: TextNode {
            value: self.get_node_text(&right_node),
            range: right_node.get_range(),
          },
          range: alias_node.get_range(),
        })
      })
      .collect()
  }

  pub(crate) fn get_node_text(&self, node: &Node) -> String {
    self
      .content
      .slice(
        self.content.byte_to_char(node.start_byte())
          ..self.content.byte_to_char(node.end_byte()),
      )
      .to_string()
  }

  pub(crate) fn get_recipes(&self) -> Vec<Recipe> {
    let root = match &self.tree {
      Some(tree) => tree.root_node(),
      None => return Vec::new(),
    };

    root
      .find_all("recipe")
      .iter()
      .filter_map(|recipe_node| {
        let recipe_name =
          self.get_node_text(&recipe_node.find("recipe_header > identifier")?);

        let dependencies = recipe_node
          .find("recipe_header > dependencies")
          .map(|dependencies_node| {
            dependencies_node
              .find_all("dependency")
              .into_iter()
              .filter_map(|dependency_node| {
                let dependency_name =
                  self.get_node_text(&dependency_node.find("identifier")?);

                let arguments = dependency_node
                  .find_all("value")
                  .iter()
                  .map(|argument_node| TextNode {
                    value: self.get_node_text(argument_node),
                    range: argument_node.get_range(),
                  })
                  .collect::<Vec<_>>();

                Some(Dependency {
                  name: dependency_name,
                  arguments,
                  range: dependency_node.get_range(),
                })
              })
              .collect::<Vec<_>>()
          })
          .unwrap_or_default();

        let parameters = recipe_node
          .find("recipe_header > parameters")
          .map_or_else(Vec::new, |params_node| {
            (0..params_node.named_child_count())
              .filter_map(|i| params_node.named_child(i))
              .filter(|param_node| {
                ["parameter", "variadic_parameter"].contains(&param_node.kind())
              })
              .filter_map(|param_node| {
                Parameter::parse(
                  &self.get_node_text(&param_node),
                  param_node.get_range(),
                )
              })
              .collect()
          });

        Some(Recipe {
          name: recipe_name,
          dependencies,
          content: self.get_node_text(recipe_node).trim().to_string(),
          parameters,
          range: recipe_node.get_range(),
        })
      })
      .collect()
  }

  pub(crate) fn get_settings(&self) -> Vec<Setting> {
    let root = match &self.tree {
      Some(tree) => tree.root_node(),
      None => return Vec::new(),
    };

    root
      .find_all("setting")
      .iter()
      .filter_map(|setting_node| {
        Setting::parse(
          &self.get_node_text(setting_node),
          setting_node.get_range(),
        )
      })
      .collect()
  }

  pub(crate) fn get_variables(&self) -> Vec<Variable> {
    let root = match &self.tree {
      Some(tree) => tree.root_node(),
      None => return Vec::new(),
    };

    root
      .find_all("assignment")
      .iter()
      .filter_map(|assignment_node| {
        let identifier_node = assignment_node.find("identifier")?;

        Some(Variable {
          name: TextNode {
            value: self.get_node_text(&identifier_node),
            range: identifier_node.get_range(),
          },
          content: self.get_node_text(assignment_node).trim().to_string(),
          range: assignment_node.get_range(),
        })
      })
      .collect()
  }

  pub(crate) fn node_at_position(
    &self,
    position: lsp::Position,
  ) -> Option<Node> {
    if let Some(tree) = &self.tree {
      let point = position.point();
      Some(tree.root_node().descendant_for_point_range(point, point)?)
    } else {
      None
    }
  }

  pub(crate) fn parse_incremental(&mut self) -> Result {
    let mut parser = Parser::new();

    let language = unsafe { tree_sitter_just() };

    parser.set_language(&language)?;

    let old_tree = self.tree.take();

    let source_code = self.content.to_string();

    self.tree = parser.parse(&source_code, old_tree.as_ref());

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::indoc,
    pretty_assertions::assert_eq,
    recipe::{ParameterKind, VariadicType},
  };

  fn document(content: &str) -> Document {
    Document::try_from(lsp::DidOpenTextDocumentParams {
      text_document: lsp::TextDocumentItem {
        uri: lsp::Url::parse("file:///test.just").unwrap(),
        language_id: "just".to_string(),
        version: 1,
        text: content.to_string(),
      },
    })
    .unwrap()
  }

  #[test]
  fn apply_change() {
    let mut doc = document(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    let original_content = doc.content.to_string();

    let change = lsp::DidChangeTextDocumentParams {
      text_document: lsp::VersionedTextDocumentIdentifier {
        uri: lsp::Url::parse("file:///test.just").unwrap(),
        version: 2,
      },
      content_changes: vec![lsp::TextDocumentContentChangeEvent {
        range: Some(lsp::Range {
          start: lsp::Position {
            line: 1,
            character: 7,
          },
          end: lsp::Position {
            line: 1,
            character: 13,
          },
        }),
        range_length: None,
        text: "\"bar\"".to_string(),
      }],
    };

    doc.apply_change(change).unwrap();

    assert_ne!(doc.content.to_string(), original_content);
    assert_eq!(doc.content.to_string(), "foo:\n  echo \"bar\"");
  }

  #[test]
  fn create_document() {
    let content = indoc! {"
      foo:
        echo foo
    "};

    let doc = document(content);

    assert_eq!(doc.content.to_string(), content);

    assert!(doc.tree.is_some());
  }

  #[test]
  fn find_nonexistent_recipe() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    assert_eq!(doc.find_recipe("nonexistent"), None);
  }

  #[test]
  fn find_recipe() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar:
        echo \"bar\"
    "});

    assert_eq!(
      doc.find_recipe("foo").unwrap(),
      Recipe {
        name: "foo".into(),
        dependencies: vec![],
        content: "foo:\n  echo \"foo\"".into(),
        parameters: vec![],
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 3,
            character: 0
          },
        }
      }
    );

    assert_eq!(
      doc.find_recipe("bar").unwrap(),
      Recipe {
        name: "bar".into(),
        dependencies: vec![],
        content: "bar:\n  echo \"bar\"".into(),
        parameters: vec![],
        range: lsp::Range {
          start: lsp::Position {
            line: 3,
            character: 0
          },
          end: lsp::Position {
            line: 5,
            character: 0
          },
        }
      }
    );

    assert!(doc.find_recipe("baz").is_none());
  }

  #[test]
  fn get_array_setting() {
    let doc = document(indoc! {
      "
      set shell := ['foo']
      "
    });

    let settings = doc.get_settings();
    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings[0],
      Setting {
        name: "shell".into(),
        kind: SettingKind::Array,
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 1,
            character: 0
          },
        }
      }
    );
  }

  #[test]
  fn get_basic_alias() {
    let doc = document(indoc! {
      "
      alias a1 := foo
      "
    });

    let aliases = doc.get_aliases();
    assert_eq!(aliases.len(), 1);

    assert_eq!(
      aliases[0],
      Alias {
        name: TextNode {
          value: "a1".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 0,
              character: 6
            },
            end: lsp::Position {
              line: 0,
              character: 8
            },
          }
        },
        value: TextNode {
          value: "foo".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 0,
              character: 12
            },
            end: lsp::Position {
              line: 0,
              character: 15
            },
          }
        },
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 0,
            character: 15
          },
        }
      }
    );
  }

  #[test]
  fn get_boolean_flag_setting() {
    let doc = document(indoc! {
      "
      set export
      "
    });

    let settings = doc.get_settings();
    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings[0],
      Setting {
        name: "export".into(),
        kind: SettingKind::Boolean,
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 1,
            character: 0
          },
        }
      }
    );
  }

  #[test]
  fn get_boolean_setting() {
    let doc = document(indoc! {
      "
      set export := true
      "
    });

    let settings = doc.get_settings();
    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings[0],
      Setting {
        name: "export".into(),
        kind: SettingKind::Boolean,
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 1,
            character: 0
          },
        }
      }
    );
  }

  #[test]
  fn get_duplicate_aliases() {
    let doc = document(indoc! {
      "
      alias duplicate := foo
      alias duplicate := bar
      "
    });

    let aliases = doc.get_aliases();
    assert_eq!(aliases.len(), 2);

    assert_eq!(
      aliases[0],
      Alias {
        name: TextNode {
          value: "duplicate".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 0,
              character: 6
            },
            end: lsp::Position {
              line: 0,
              character: 15
            },
          }
        },
        value: TextNode {
          value: "foo".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 0,
              character: 19
            },
            end: lsp::Position {
              line: 0,
              character: 22
            },
          }
        },
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 0,
            character: 22
          },
        }
      }
    );

    assert_eq!(
      aliases[1],
      Alias {
        name: TextNode {
          value: "duplicate".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 1,
              character: 6
            },
            end: lsp::Position {
              line: 1,
              character: 15
            },
          }
        },
        value: TextNode {
          value: "bar".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 1,
              character: 19
            },
            end: lsp::Position {
              line: 1,
              character: 22
            },
          }
        },
        range: lsp::Range {
          start: lsp::Position {
            line: 1,
            character: 0
          },
          end: lsp::Position {
            line: 1,
            character: 22
          },
        }
      }
    );
  }

  #[test]
  fn get_multiple_aliases() {
    let doc = document(indoc! {
      "
      alias a1 := foo
      alias a2 := bar
      "
    });

    let aliases = doc.get_aliases();
    assert_eq!(aliases.len(), 2);

    assert_eq!(
      aliases[0],
      Alias {
        name: TextNode {
          value: "a1".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 0,
              character: 6
            },
            end: lsp::Position {
              line: 0,
              character: 8
            },
          }
        },
        value: TextNode {
          value: "foo".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 0,
              character: 12
            },
            end: lsp::Position {
              line: 0,
              character: 15
            },
          }
        },
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 0,
            character: 15
          },
        }
      }
    );

    assert_eq!(
      aliases[1],
      Alias {
        name: TextNode {
          value: "a2".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 1,
              character: 6
            },
            end: lsp::Position {
              line: 1,
              character: 8
            },
          }
        },
        value: TextNode {
          value: "bar".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 1,
              character: 12
            },
            end: lsp::Position {
              line: 1,
              character: 15
            },
          }
        },
        range: lsp::Range {
          start: lsp::Position {
            line: 1,
            character: 0
          },
          end: lsp::Position {
            line: 1,
            character: 15
          },
        }
      }
    );
  }

  #[test]
  fn get_multiple_settings() {
    let doc = document(indoc! {
      "
      set export := true
      set shell := ['foo']
      set bar := 'wow!'
      "
    });

    let settings = doc.get_settings();
    assert_eq!(settings.len(), 3);

    assert_eq!(
      settings[0],
      Setting {
        name: "export".into(),
        kind: SettingKind::Boolean,
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 1,
            character: 0
          },
        }
      }
    );

    assert_eq!(
      settings[1],
      Setting {
        name: "shell".into(),
        kind: SettingKind::Array,
        range: lsp::Range {
          start: lsp::Position {
            line: 1,
            character: 0
          },
          end: lsp::Position {
            line: 2,
            character: 0
          },
        }
      }
    );

    assert_eq!(
      settings[2],
      Setting {
        name: "bar".into(),
        kind: SettingKind::String,
        range: lsp::Range {
          start: lsp::Position {
            line: 2,
            character: 0
          },
          end: lsp::Position {
            line: 3,
            character: 0
          },
        }
      }
    );
  }

  #[test]
  fn get_string_setting() {
    let doc = document(indoc! {
      "
      set bar := 'wow!'
      "
    });

    let settings = doc.get_settings();
    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings[0],
      Setting {
        name: "bar".into(),
        kind: SettingKind::String,
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 1,
            character: 0
          },
        }
      }
    );
  }

  #[test]
  fn get_variables() {
    let doc = document(indoc! {
      "
      tmpdir  := `mktemp -d`
      version := \"0.2.7\"
      tardir  := tmpdir / \"awesomesauce-\" + version
      tarball := tardir + \".tar.gz\"
      config  := quote(config_dir() / \".project-config\")
      "
    });

    assert_eq!(
      doc.get_variables(),
      vec![
        Variable {
          name: TextNode {
            value: "tmpdir".into(),
            range: lsp::Range {
              start: lsp::Position {
                line: 0,
                character: 0,
              },
              end: lsp::Position {
                line: 0,
                character: 6,
              },
            },
          },
          content: "tmpdir  := `mktemp -d`".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 0,
              character: 0,
            },
            end: lsp::Position {
              line: 1,
              character: 0,
            },
          },
        },
        Variable {
          name: TextNode {
            value: "version".into(),
            range: lsp::Range {
              start: lsp::Position {
                line: 1,
                character: 0,
              },
              end: lsp::Position {
                line: 1,
                character: 7,
              },
            },
          },
          content: "version := \"0.2.7\"".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 1,
              character: 0,
            },
            end: lsp::Position {
              line: 2,
              character: 0,
            },
          },
        },
        Variable {
          name: TextNode {
            value: "tardir".into(),
            range: lsp::Range {
              start: lsp::Position {
                line: 2,
                character: 0,
              },
              end: lsp::Position {
                line: 2,
                character: 6,
              },
            },
          },
          content: "tardir  := tmpdir / \"awesomesauce-\" + version".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 2,
              character: 0,
            },
            end: lsp::Position {
              line: 3,
              character: 0,
            },
          },
        },
        Variable {
          name: TextNode {
            value: "tarball".into(),
            range: lsp::Range {
              start: lsp::Position {
                line: 3,
                character: 0,
              },
              end: lsp::Position {
                line: 3,
                character: 7,
              },
            },
          },
          content: "tarball := tardir + \".tar.gz\"".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 3,
              character: 0,
            },
            end: lsp::Position {
              line: 4,
              character: 0,
            },
          },
        },
        Variable {
          name: TextNode {
            value: "config".into(),
            range: lsp::Range {
              start: lsp::Position {
                line: 4,
                character: 0,
              },
              end: lsp::Position {
                line: 4,
                character: 6,
              },
            },
          },
          content: "config  := quote(config_dir() / \".project-config\")"
            .into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 4,
              character: 0,
            },
            end: lsp::Position {
              line: 5,
              character: 0,
            },
          },
        },
      ]
    );
  }

  #[test]
  fn multiple_recipes() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar:
        echo \"bar\"
      "
    });

    assert_eq!(
      doc.find_recipe("foo"),
      Some(Recipe {
        name: "foo".into(),
        dependencies: vec![],
        parameters: vec![],
        content: "foo:\n  echo \"foo\"".into(),
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 3,
            character: 0
          },
        }
      })
    );

    assert_eq!(
      doc.find_recipe("bar"),
      Some(Recipe {
        name: "bar".into(),
        dependencies: vec![],
        parameters: vec![],
        content: "bar:\n  echo \"bar\"".into(),
        range: lsp::Range {
          start: lsp::Position {
            line: 3,
            character: 0
          },
          end: lsp::Position {
            line: 5,
            character: 0
          },
        }
      })
    );
  }

  #[test]
  fn node_at_position() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
    "});

    let node = doc
      .node_at_position(lsp::Position {
        line: 1,
        character: 1,
      })
      .unwrap();

    assert_eq!(node.kind(), "recipe");
    assert_eq!(doc.get_node_text(&node), "foo:\n  echo \"foo\"\n\n");

    let node = doc
      .node_at_position(lsp::Position {
        line: 4,
        character: 6,
      })
      .unwrap();

    assert_eq!(node.kind(), "text");
    assert_eq!(doc.get_node_text(&node), "echo \"bar\"");
  }

  #[test]
  fn recipe_with_default_parameter() {
    let doc = document(indoc! {
      "
      baz first second=\"default\":
        echo \"{{first}} {{second}}\"
      "
    });

    assert_eq!(
      doc.find_recipe("baz"),
      Some(Recipe {
        name: "baz".into(),
        dependencies: vec![],
        parameters: vec![
          Parameter {
            name: "first".into(),
            kind: ParameterKind::Normal,
            default_value: None,
            content: "first".into(),
            range: lsp::Range {
              start: lsp::Position {
                line: 0,
                character: 4
              },
              end: lsp::Position {
                line: 0,
                character: 9
              },
            }
          },
          Parameter {
            name: "second".into(),
            kind: ParameterKind::Normal,
            default_value: Some("\"default\"".into()),
            content: "second=\"default\"".into(),
            range: lsp::Range {
              start: lsp::Position {
                line: 0,
                character: 10
              },
              end: lsp::Position {
                line: 0,
                character: 26
              },
            }
          }
        ],
        content:
          "baz first second=\"default\":\n  echo \"{{first}} {{second}}\""
            .into(),
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 2,
            character: 0
          },
        }
      })
    );
  }

  #[test]
  fn recipe_with_dependency() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
      "
    });

    assert_eq!(
      doc.find_recipe("bar"),
      Some(Recipe {
        name: "bar".into(),
        dependencies: vec![Dependency {
          name: "foo".into(),
          arguments: vec![],
          range: lsp::Range {
            start: lsp::Position {
              line: 3,
              character: 5
            },
            end: lsp::Position {
              line: 3,
              character: 8
            },
          }
        }],
        parameters: vec![],
        content: "bar: foo\n  echo \"bar\"".into(),
        range: lsp::Range {
          start: lsp::Position {
            line: 3,
            character: 0
          },
          end: lsp::Position {
            line: 5,
            character: 0
          },
        }
      })
    );
  }

  #[test]
  fn recipe_with_dependency_arguments() {
    let doc = document(indoc! {
      "
      foo arg1 arg2:
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo 'value1' 'value2')
        echo \"bar\"
      "
    });

    assert_eq!(
      doc.find_recipe("bar"),
      Some(Recipe {
        name: "bar".into(),
        dependencies: vec![Dependency {
          name: "foo".into(),
          arguments: vec![
            TextNode {
              value: "'value1'".into(),
              range: lsp::Range {
                start: lsp::Position {
                  line: 3,
                  character: 10
                },
                end: lsp::Position {
                  line: 3,
                  character: 18
                },
              }
            },
            TextNode {
              value: "'value2'".into(),
              range: lsp::Range {
                start: lsp::Position {
                  line: 3,
                  character: 19
                },
                end: lsp::Position {
                  line: 3,
                  character: 27
                },
              }
            }
          ],
          range: lsp::Range {
            start: lsp::Position {
              line: 3,
              character: 5
            },
            end: lsp::Position {
              line: 3,
              character: 28
            },
          }
        }],
        parameters: vec![],
        content: "bar: (foo 'value1' 'value2')\n  echo \"bar\"".into(),
        range: lsp::Range {
          start: lsp::Position {
            line: 3,
            character: 0
          },
          end: lsp::Position {
            line: 5,
            character: 0
          },
        }
      })
    );
  }

  #[test]
  fn recipe_with_multiple_dependencies() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar:
        echo \"bar\"

      baz: foo bar
        echo \"baz\"
      "
    });

    assert_eq!(
      doc.find_recipe("baz"),
      Some(Recipe {
        name: "baz".into(),
        dependencies: vec![
          Dependency {
            name: "foo".into(),
            arguments: vec![],
            range: lsp::Range {
              start: lsp::Position {
                line: 6,
                character: 5
              },
              end: lsp::Position {
                line: 6,
                character: 8
              },
            }
          },
          Dependency {
            name: "bar".into(),
            arguments: vec![],
            range: lsp::Range {
              start: lsp::Position {
                line: 6,
                character: 9
              },
              end: lsp::Position {
                line: 6,
                character: 12
              },
            }
          }
        ],
        parameters: vec![],
        content: "baz: foo bar\n  echo \"baz\"".into(),
        range: lsp::Range {
          start: lsp::Position {
            line: 6,
            character: 0
          },
          end: lsp::Position {
            line: 8,
            character: 0
          },
        }
      })
    );
  }

  #[test]
  fn recipe_with_parameters() {
    let doc = document(indoc! {
      "
      bar target $lol:
        echo \"Building {{target}}\"
      "
    });

    assert_eq!(
      doc.find_recipe("bar"),
      Some(Recipe {
        name: "bar".into(),
        dependencies: vec![],
        parameters: vec![
          Parameter {
            name: "target".into(),
            kind: ParameterKind::Normal,
            default_value: None,
            content: "target".into(),
            range: lsp::Range {
              start: lsp::Position {
                line: 0,
                character: 4
              },
              end: lsp::Position {
                line: 0,
                character: 10
              },
            }
          },
          Parameter {
            name: "lol".into(),
            kind: ParameterKind::Export,
            default_value: None,
            content: "$lol".into(),
            range: lsp::Range {
              start: lsp::Position {
                line: 0,
                character: 11
              },
              end: lsp::Position {
                line: 0,
                character: 15
              },
            }
          }
        ],
        content: "bar target $lol:\n  echo \"Building {{target}}\"".into(),
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 2,
            character: 0
          },
        }
      })
    );
  }

  #[test]
  fn recipe_with_variadic_parameter() {
    let doc = document(indoc! {
      "
      baz first +second=\"default\":
        echo \"{{first}} {{second}}\"
      "
    });

    assert_eq!(
      doc.find_recipe("baz"),
      Some(Recipe {
        name: "baz".into(),
        dependencies: vec![],
        parameters: vec![
          Parameter {
            name: "first".into(),
            kind: ParameterKind::Normal,
            default_value: None,
            content: "first".into(),
            range: lsp::Range {
              start: lsp::Position {
                line: 0,
                character: 4
              },
              end: lsp::Position {
                line: 0,
                character: 9
              },
            }
          },
          Parameter {
            name: "second".into(),
            kind: ParameterKind::Variadic(VariadicType::OneOrMore),
            default_value: Some("\"default\"".into()),
            content: "+second=\"default\"".into(),
            range: lsp::Range {
              start: lsp::Position {
                line: 0,
                character: 10
              },
              end: lsp::Position {
                line: 0,
                character: 27
              },
            }
          }
        ],
        content:
          "baz first +second=\"default\":\n  echo \"{{first}} {{second}}\""
            .into(),
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 2,
            character: 0
          },
        }
      })
    );
  }

  #[test]
  fn recipe_without_parameters_or_dependencies() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    assert_eq!(
      doc.find_recipe("foo"),
      Some(Recipe {
        name: "foo".into(),
        dependencies: vec![],
        parameters: vec![],
        content: "foo:\n  echo \"foo\"".into(),
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 2,
            character: 0
          },
        }
      })
    );
  }
}
