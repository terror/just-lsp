use super::*;

#[derive(Debug)]
pub(crate) struct Document {
  pub(crate) content: Rope,
  pub(crate) tree: Option<Tree>,
  pub(crate) uri: lsp::Url,
  pub(crate) version: i32,
}

impl TryFrom<lsp::DidOpenTextDocumentParams> for Document {
  type Error = Error;

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

    document.parse()?;

    Ok(document)
  }
}

impl Document {
  #[must_use]
  pub(crate) fn aliases(&self) -> Vec<Alias> {
    self.tree.as_ref().map_or(Vec::new(), |tree| {
      tree
        .root_node()
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
    })
  }

  /// Applies incremental edits from the client and reparses the syntax tree.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] if tree-sitter fails to parse the updated document.
  pub(crate) fn apply_change(
    &mut self,
    params: lsp::DidChangeTextDocumentParams,
  ) -> Result {
    let lsp::DidChangeTextDocumentParams {
      content_changes,
      text_document: lsp::VersionedTextDocumentIdentifier { version, .. },
      ..
    } = params;

    self.version = version;

    for change in content_changes {
      let edit = self.content.build_edit(&change);

      self.content.apply_edit(&edit);

      if let Some(tree) = &mut self.tree {
        tree.edit(&edit.input_edit);
      }
    }

    self.parse()?;

    Ok(())
  }

  #[must_use]
  pub(crate) fn attributes(&self) -> Vec<Attribute> {
    self.tree.as_ref().map_or(Vec::new(), |tree| {
      tree
        .root_node()
        .find_all("attribute")
        .into_iter()
        .flat_map(|attribute_node| {
          let target = attribute_node
            .parent()
            .and_then(|parent| AttributeTarget::try_from_kind(parent.kind()));

          attribute_node
            .find_all("identifier")
            .into_iter()
            .map(move |identifier_node| {
              let arguments = identifier_node
                .find_siblings_until("string", "identifier")
                .into_iter()
                .map(|argument_node| TextNode {
                  value: self.get_node_text(&argument_node),
                  range: argument_node.get_range(),
                })
                .collect::<Vec<_>>();

              Attribute {
                name: TextNode {
                  value: self.get_node_text(&identifier_node),
                  range: identifier_node.get_range(),
                },
                arguments,
                target,
                range: attribute_node.get_range(),
              }
            })
            .collect::<Vec<_>>()
        })
        .collect()
    })
  }

  #[must_use]
  pub(crate) fn find_recipe(&self, name: &str) -> Option<Recipe> {
    self
      .recipes()
      .into_iter()
      .find(|recipe| recipe.name == name)
  }

  #[must_use]
  pub(crate) fn find_variable(&self, name: &str) -> Option<Variable> {
    self
      .variables()
      .into_iter()
      .find(|var| var.name.value == name)
  }

  #[must_use]
  pub(crate) fn function_calls(&self) -> Vec<FunctionCall> {
    self.tree.as_ref().map_or(Vec::new(), |tree| {
      tree
        .root_node()
        .find_all("function_call")
        .into_iter()
        .filter_map(|function_call_node| {
          let identifier_node = function_call_node.find("identifier")?;

          let arguments = function_call_node
            .find("sequence")
            .map(|sequence| {
              sequence
                .find_all("^expression")
                .into_iter()
                .map(|argument_node| TextNode {
                  value: self.get_node_text(&argument_node),
                  range: argument_node.get_range(),
                })
                .collect::<Vec<_>>()
            })
            .unwrap_or_default();

          Some(FunctionCall {
            name: TextNode {
              value: self.get_node_text(&identifier_node),
              range: identifier_node.get_range(),
            },
            arguments,
            range: function_call_node.get_range(),
          })
        })
        .collect()
    })
  }

  #[must_use]
  pub(crate) fn get_node_text(&self, node: &Node) -> String {
    self
      .content
      .slice(
        self.content.byte_to_char(node.start_byte())
          ..self.content.byte_to_char(node.end_byte()),
      )
      .to_string()
  }

  #[must_use]
  pub(crate) fn node_at_position(
    &self,
    position: lsp::Position,
  ) -> Option<Node<'_>> {
    if let Some(tree) = &self.tree {
      let point = position.point();
      Some(tree.root_node().descendant_for_point_range(point, point)?)
    } else {
      None
    }
  }

  /// Parses the current document contents and updates the cached syntax tree.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] if the tree-sitter parser cannot be created or the
  /// contents fail to parse.
  pub(crate) fn parse(&mut self) -> Result {
    let mut parser = Parser::new();

    // SAFETY: tree_sitter_just returns a static language definition.
    parser.set_language(&unsafe { tree_sitter_just() })?;

    let old_tree = self.tree.take();

    self.tree = parser.parse(self.content.to_string(), old_tree.as_ref());

    Ok(())
  }

  #[must_use]
  pub(crate) fn recipes(&self) -> Vec<Recipe> {
    self.tree.as_ref().map_or(Vec::new(), |tree| {
      tree
        .root_node()
        .find_all("recipe")
        .iter()
        .filter_map(|recipe_node| {
          let recipe_name = self
            .get_node_text(&recipe_node.find("recipe_header > identifier")?);

          let attributes = recipe_node
            .find_all("attribute")
            .iter()
            .filter_map(|attribute_node| {
              let identifier = attribute_node.find("identifier")?;

              let arguments = attribute_node
                .find_all("string")
                .iter()
                .map(|argument_node| TextNode {
                  value: self.get_node_text(argument_node),
                  range: argument_node.get_range(),
                })
                .collect::<Vec<_>>();

              Some(Attribute {
                name: TextNode {
                  value: self.get_node_text(&identifier),
                  range: identifier.get_range(),
                },
                arguments,
                target: Some(AttributeTarget::Recipe),
                range: attribute_node.get_range(),
              })
            })
            .collect::<Vec<_>>();

          let dependencies = recipe_node
            .find("recipe_header > dependencies")
            .map(|dependencies_node| {
              dependencies_node
                .find_all("dependency")
                .into_iter()
                .filter_map(|dependency_node| {
                  let dependency_name =
                    self.get_node_text(&dependency_node.find("identifier")?);

                  let arguments = if let Some(dep_expr_node) =
                    dependency_node.find("dependency_expression")
                  {
                    dep_expr_node
                      .find_all("^expression")
                      .iter()
                      .map(|argument_node| TextNode {
                        value: self.get_node_text(argument_node),
                        range: argument_node.get_range(),
                      })
                      .collect::<Vec<_>>()
                  } else {
                    vec![]
                  };

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
            .map_or_else(Vec::new, |parameters_node| {
              parameters_node
                .find_all("^parameter, ^variadic_parameter")
                .iter()
                .filter_map(|param_node| {
                  Parameter::parse(
                    &self.get_node_text(param_node),
                    param_node.get_range(),
                  )
                })
                .collect()
            });

          Some(Recipe {
            name: recipe_name,
            attributes,
            dependencies,
            content: self.get_node_text(recipe_node).trim().to_string(),
            parameters,
            range: recipe_node.get_range(),
          })
        })
        .collect()
    })
  }

  #[must_use]
  pub(crate) fn settings(&self) -> Vec<Setting> {
    self.tree.as_ref().map_or(Vec::new(), |tree| {
      tree
        .root_node()
        .find_all("setting")
        .iter()
        .filter_map(|setting_node| {
          Setting::parse(
            &self.get_node_text(setting_node),
            setting_node.get_range(),
          )
        })
        .collect()
    })
  }

  #[must_use]
  pub(crate) fn variables(&self) -> Vec<Variable> {
    self.tree.as_ref().map_or(Vec::new(), |tree| {
      tree
        .root_node()
        .find_all("assignment")
        .iter()
        .filter_map(|assignment_node| {
          let identifier_node = assignment_node.child_by_field_name("left")?;

          Some(Variable {
            name: TextNode {
              value: self.get_node_text(&identifier_node),
              range: identifier_node.get_range(),
            },
            export: identifier_node.get_parent("export").is_some(),
            content: self.get_node_text(assignment_node).trim().to_string(),
            range: assignment_node.get_range(),
          })
        })
        .collect()
    })
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*, indoc::indoc, parameter::VariadicType,
    pretty_assertions::assert_eq,
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
        attributes: vec![],
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
        attributes: vec![],
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

    let settings = doc.settings();
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

    let aliases = doc.aliases();
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

    let settings = doc.settings();
    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings[0],
      Setting {
        name: "export".into(),
        kind: SettingKind::Boolean(true),
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

    let settings = doc.settings();
    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings[0],
      Setting {
        name: "export".into(),
        kind: SettingKind::Boolean(true),
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

    let aliases = doc.aliases();
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

    let aliases = doc.aliases();
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

    let settings = doc.settings();
    assert_eq!(settings.len(), 3);

    assert_eq!(
      settings[0],
      Setting {
        name: "export".into(),
        kind: SettingKind::Boolean(true),
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

    let settings = doc.settings();
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
      export EDITOR := 'nvim'
      "
    });

    assert_eq!(
      doc.variables(),
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
          export: false,
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
          export: false,
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
          export: false,
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
          export: false,
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
          export: false,
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
        Variable {
          name: TextNode {
            value: "EDITOR".into(),
            range: lsp::Range {
              start: lsp::Position {
                line: 5,
                character: 7,
              },
              end: lsp::Position {
                line: 5,
                character: 13,
              },
            },
          },
          export: true,
          content: "EDITOR := 'nvim'".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 5,
              character: 7,
            },
            end: lsp::Position {
              line: 6,
              character: 0,
            },
          },
        },
      ]
    );
  }

  #[test]
  fn private_exported_variable_is_marked_exported() {
    let doc = document(indoc! {
      "
      [private]
      export PATH := '/usr/local/bin'
      "
    });

    let variables = doc.variables();

    assert!(variables[0].export);

    assert_eq!(variables.len(), 1);
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
        attributes: vec![],
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
        attributes: vec![],
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
        attributes: vec![],
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
        attributes: vec![],
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
        attributes: vec![],
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
        attributes: vec![],
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
        attributes: vec![],
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
        attributes: vec![],
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
        attributes: vec![],
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

  #[test]
  fn recipe_with_attributes() {
    let doc = document(indoc! {
      "
      [private]
      [description: \"This is a test recipe\"]
      [tags(\"test\", \"example\")]
      foo:
        echo \"foo\"
      "
    });

    let recipe = doc.find_recipe("foo").unwrap();

    assert_eq!(recipe.attributes.len(), 3);

    assert_eq!(
      recipe.attributes[0],
      Attribute {
        name: TextNode {
          value: "private".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 0,
              character: 1
            },
            end: lsp::Position {
              line: 0,
              character: 8
            },
          }
        },
        arguments: vec![],
        target: Some(AttributeTarget::Recipe),
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
      recipe.attributes[1],
      Attribute {
        name: TextNode {
          value: "description".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 1,
              character: 1
            },
            end: lsp::Position {
              line: 1,
              character: 12
            },
          }
        },
        arguments: vec![TextNode {
          value: "\"This is a test recipe\"".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 1,
              character: 14
            },
            end: lsp::Position {
              line: 1,
              character: 37
            },
          }
        }],
        target: Some(AttributeTarget::Recipe),
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
      recipe.attributes[2],
      Attribute {
        name: TextNode {
          value: "tags".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 2,
              character: 1
            },
            end: lsp::Position {
              line: 2,
              character: 5
            },
          }
        },
        arguments: vec![
          TextNode {
            value: "\"test\"".into(),
            range: lsp::Range {
              start: lsp::Position {
                line: 2,
                character: 6
              },
              end: lsp::Position {
                line: 2,
                character: 12
              },
            }
          },
          TextNode {
            value: "\"example\"".into(),
            range: lsp::Range {
              start: lsp::Position {
                line: 2,
                character: 14
              },
              end: lsp::Position {
                line: 2,
                character: 23
              },
            }
          }
        ],
        target: Some(AttributeTarget::Recipe),
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
  fn list_document_attributes() {
    let doc = document(indoc! {
      "
      [private, description: \"desc\"]
      foo:
        echo \"foo\"

      [alias_attr]
      alias build := foo

      [var_attr(\"value\")]
      bar := \"bar\"

      [export_attr]
      export baz := \"baz\"

      [module_attr]
      mod utils \"./utils.just\"
      "
    });

    let attributes = doc.attributes();

    let names_and_targets = attributes
      .iter()
      .map(|attr| (attr.name.value.clone(), attr.target))
      .collect::<Vec<_>>();

    assert_eq!(
      names_and_targets,
      vec![
        ("private".into(), Some(AttributeTarget::Recipe)),
        ("description".into(), Some(AttributeTarget::Recipe)),
        ("alias_attr".into(), Some(AttributeTarget::Alias)),
        ("var_attr".into(), Some(AttributeTarget::Assignment)),
        ("export_attr".into(), Some(AttributeTarget::Assignment)),
        ("module_attr".into(), Some(AttributeTarget::Module)),
      ]
    );

    assert_eq!(attributes[1].arguments.len(), 1);
    assert_eq!(attributes[1].arguments[0].value, "\"desc\"");
    assert_eq!(attributes[3].arguments.len(), 1);
    assert_eq!(attributes[3].arguments[0].value, "\"value\"");
  }

  #[test]
  fn list_function_calls() {
    let doc = document(indoc! {"
      foo:
        echo {{arch()}}
        echo {{env_var(\"HOME\", \"fallback\")}}
    "});

    let calls = doc.function_calls();

    assert_eq!(calls.len(), 2);

    assert_eq!(calls[0].name.value, "arch");
    assert_eq!(calls[0].arguments.len(), 0);

    assert_eq!(calls[1].name.value, "env_var");
    assert_eq!(calls[1].arguments.len(), 2);
    assert_eq!(calls[1].arguments[0].value, "\"HOME\"");
    assert_eq!(calls[1].arguments[1].value, "\"fallback\"");
  }
}
