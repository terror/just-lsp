use super::*;
use crate::position_ext::PositionExt;

#[derive(Debug)]
pub(crate) struct Document {
  pub(crate) content: Rope,
  pub(crate) tree: Option<Tree>,
  pub(crate) uri: lsp::Url,
  pub(crate) version: i32,
}

#[cfg(test)]
impl From<&str> for Document {
  fn from(value: &str) -> Self {
    let mut document = Self {
      content: value.into(),
      tree: None,
      uri: lsp::Url::parse("file:///test.just").unwrap(),
      version: 1,
    };

    document.parse().unwrap();

    document
  }
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
          let left_node = alias_node.child_by_field_name("left")?;
          let right_node = alias_node.child_by_field_name("right")?;

          Some(Alias {
            name: TextNode {
              value: self.get_node_text(&left_node),
              range: left_node.get_range(self),
            },
            value: TextNode {
              value: self.get_node_text(&right_node),
              range: right_node.get_range(self),
            },
            range: alias_node.get_range(self),
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
            .find_all("^identifier")
            .into_iter()
            .map(move |identifier_node| {
              let arguments = identifier_node
                .find_siblings_until("string", "identifier")
                .into_iter()
                .map(|argument_node| TextNode {
                  value: self.get_node_text(&argument_node),
                  range: argument_node.get_range(self),
                })
                .collect::<Vec<_>>();

              Attribute {
                name: TextNode {
                  value: self.get_node_text(&identifier_node),
                  range: identifier_node.get_range(self),
                },
                arguments,
                target,
                range: attribute_node.get_range(self),
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
      .find(|recipe| recipe.name.value == name)
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
                  range: argument_node.get_range(self),
                })
                .collect::<Vec<_>>()
            })
            .unwrap_or_default();

          Some(FunctionCall {
            name: TextNode {
              value: self.get_node_text(&identifier_node),
              range: identifier_node.get_range(self),
            },
            arguments,
            range: function_call_node.get_range(self),
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
  pub(crate) fn imports(&self) -> Vec<Import> {
    self.tree.as_ref().map_or(Vec::new(), |tree| {
      tree
        .root_node()
        .find_all("import")
        .iter()
        .filter_map(|import_node| {
          let string_node = import_node.find("string")?;

          let optional = (0..import_node.child_count()).any(|index| {
            u32::try_from(index)
              .ok()
              .and_then(|i| import_node.child(i))
              .is_some_and(|child| child.kind() == "?")
          });

          let raw = self.get_node_text(&string_node);

          let path = raw
            .strip_prefix(['\'', '"'])
            .unwrap_or(&raw)
            .strip_suffix(['\'', '"'])
            .unwrap_or(&raw)
            .to_string();

          Some(Import { optional, path })
        })
        .collect()
    })
  }

  /// Returns the syntax tree node at the given LSP `Position`.
  #[must_use]
  pub(crate) fn node_at_position(
    &self,
    position: lsp::Position,
  ) -> Option<Node<'_>> {
    let tree = self.tree.as_ref()?;
    let point = position.point(self);
    tree.root_node().descendant_for_point_range(point, point)
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
          let name_node = recipe_node.find("recipe_header > identifier")?;

          let recipe_name = TextNode {
            value: self.get_node_text(&name_node),
            range: name_node.get_range(self),
          };

          let attributes = recipe_node
            .find_all("attribute")
            .into_iter()
            .flat_map(|attribute_node| {
              attribute_node
                .find_all("^identifier")
                .into_iter()
                .map(|identifier_node| {
                  let arguments = identifier_node
                    .find_siblings_until("string", "identifier")
                    .into_iter()
                    .map(|argument_node| TextNode {
                      value: self.get_node_text(&argument_node),
                      range: argument_node.get_range(self),
                    })
                    .collect::<Vec<_>>();

                  Attribute {
                    name: TextNode {
                      value: self.get_node_text(&identifier_node),
                      range: identifier_node.get_range(self),
                    },
                    arguments,
                    target: Some(AttributeTarget::Recipe),
                    range: attribute_node.get_range(self),
                  }
                })
                .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

          let dependencies = recipe_node
            .find("recipe_header > dependencies")
            .map(|dependencies_node| {
              dependencies_node
                .find_all("dependency")
                .into_iter()
                .filter_map(|dependency_node| {
                  let dependency_name = dependency_node
                    .child_by_field_name("name")
                    .or_else(|| {
                      dependency_node
                        .find("dependency_expression")
                        .and_then(|node| node.child_by_field_name("name"))
                    })
                    .map(|node| self.get_node_text(&node))?;

                  let arguments = dependency_node
                    .find("dependency_expression")
                    .map(|dependency_expression_node| {
                      dependency_expression_node
                        .find_all("^expression")
                        .iter()
                        .map(|argument_node| TextNode {
                          value: self.get_node_text(argument_node),
                          range: argument_node.get_range(self),
                        })
                        .collect()
                    })
                    .unwrap_or_default();

                  Some(Dependency {
                    name: dependency_name,
                    arguments,
                    range: dependency_node.get_range(self),
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
                .filter_map(|parameter_node| {
                  Parameter::parse(
                    &self.get_node_text(parameter_node),
                    parameter_node.get_range(self),
                  )
                })
                .collect()
            });

          let shebang =
            recipe_node
              .find("recipe_body > shebang")
              .map(|shebang_node| TextNode {
                value: self.get_node_text(&shebang_node),
                range: shebang_node.get_range(self),
              });

          Some(Recipe {
            name: recipe_name,
            attributes,
            dependencies,
            content: self.get_node_text(recipe_node).trim().to_string(),
            parameters,
            range: recipe_node.get_range(self),
            shebang,
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
            setting_node.get_range(self),
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
              range: identifier_node.get_range(self),
            },
            export: identifier_node.get_parent("export").is_some(),
            unexport: identifier_node.get_parent("unexport").is_some(),
            content: self.get_node_text(assignment_node).trim().to_string(),
            range: assignment_node.get_range(self),
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

  type RangeSpec = (u32, u32, u32, u32);

  fn range(
    (start_line, start_character, end_line, end_character): RangeSpec,
  ) -> lsp::Range {
    lsp::Range {
      start: lsp::Position {
        line: start_line,
        character: start_character,
      },
      end: lsp::Position {
        line: end_line,
        character: end_character,
      },
    }
  }

  #[test]
  fn basic_import() {
    let document = Document::from(indoc! {"
      import 'foo.just'
    "});

    assert_eq!(
      document.imports(),
      vec![Import {
        optional: false,
        path: "foo.just".into(),
      }]
    );
  }

  #[test]
  fn optional_import() {
    let document = Document::from(indoc! {"
      import? 'bar.just'
    "});

    assert_eq!(
      document.imports(),
      vec![Import {
        optional: true,
        path: "bar.just".into(),
      }]
    );
  }

  #[test]
  fn create_document() {
    let content = indoc! {"
      foo:
        echo foo
    "};

    let document = Document::from(content);

    assert_eq!(document.content.to_string(), content);

    assert!(document.tree.is_some());
  }

  #[test]
  fn apply_change() {
    let mut document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    let original_content = document.content.to_string();

    let change = lsp::DidChangeTextDocumentParams {
      text_document: lsp::VersionedTextDocumentIdentifier {
        uri: lsp::Url::parse("file:///test.just").unwrap(),
        version: 2,
      },
      content_changes: vec![lsp::TextDocumentContentChangeEvent {
        range: Some(range((1, 7, 1, 13))),
        range_length: None,
        text: "\"bar\"".to_string(),
      }],
    };

    document.apply_change(change).unwrap();

    assert_ne!(document.content.to_string(), original_content);
    assert_eq!(document.content.to_string(), "foo:\n  echo \"bar\"");
  }

  #[test]
  fn find_nonexistent_recipe() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    assert_eq!(document.find_recipe("nonexistent"), None);
  }

  #[test]
  fn find_recipe() {
    let document = Document::from(indoc! {"
      foo:
        echo \"foo\"

      bar:
        echo \"bar\"
    "});

    assert_eq!(
      document.find_recipe("foo").unwrap(),
      Recipe {
        name: TextNode {
          value: "foo".into(),
          range: range((0, 0, 0, 3))
        },
        attributes: vec![],
        dependencies: vec![],
        content: "foo:\n  echo \"foo\"".into(),
        parameters: vec![],
        range: range((0, 0, 3, 0)),
        shebang: None,
      }
    );

    assert_eq!(
      document.find_recipe("bar").unwrap(),
      Recipe {
        name: TextNode {
          value: "bar".into(),
          range: range((3, 0, 3, 3))
        },
        attributes: vec![],
        dependencies: vec![],
        content: "bar:\n  echo \"bar\"".into(),
        parameters: vec![],
        range: range((3, 0, 5, 0)),
        shebang: None,
      }
    );

    assert!(document.find_recipe("baz").is_none());
  }

  #[test]
  fn get_array_setting() {
    let document = Document::from(indoc! {
      "
      set shell := ['foo']
      "
    });

    let settings = document.settings();

    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings,
      vec![Setting {
        name: "shell".into(),
        kind: SettingKind::Array,
        range: range((0, 0, 1, 0))
      }]
    );
  }

  #[test]
  fn get_basic_alias() {
    let document = Document::from(indoc! {
      "
      alias a1 := foo
      "
    });

    let aliases = document.aliases();

    assert_eq!(aliases.len(), 1);

    assert_eq!(
      aliases,
      vec![Alias {
        name: TextNode {
          value: "a1".into(),
          range: range((0, 6, 0, 8))
        },
        value: TextNode {
          value: "foo".into(),
          range: range((0, 12, 0, 15))
        },
        range: range((0, 0, 0, 15))
      }]
    );
  }

  #[test]
  fn get_alias_with_module_path() {
    let document = Document::from(indoc! {
      "
      alias a1 := tools::build
      "
    });

    let aliases = document.aliases();

    assert_eq!(aliases.len(), 1);

    assert_eq!(
      aliases,
      vec![Alias {
        name: TextNode {
          value: "a1".into(),
          range: range((0, 6, 0, 8))
        },
        value: TextNode {
          value: "tools::build".into(),
          range: range((0, 12, 0, 24))
        },
        range: range((0, 0, 0, 24))
      }]
    );
  }

  #[test]
  fn get_boolean_flag_setting() {
    let document = Document::from(indoc! {
      "
      set export
      "
    });

    let settings = document.settings();

    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings,
      vec![Setting {
        name: "export".into(),
        kind: SettingKind::Boolean(true),
        range: range((0, 0, 1, 0))
      }]
    );
  }

  #[test]
  fn get_boolean_setting() {
    let document = Document::from(indoc! {
      "
      set export := true
      "
    });

    let settings = document.settings();

    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings,
      vec![Setting {
        name: "export".into(),
        kind: SettingKind::Boolean(true),
        range: range((0, 0, 1, 0))
      }]
    );
  }

  #[test]
  fn get_duplicate_aliases() {
    let document = Document::from(indoc! {
      "
      alias duplicate := foo
      alias duplicate := bar
      "
    });

    let aliases = document.aliases();

    assert_eq!(aliases.len(), 2);

    assert_eq!(
      aliases,
      vec![
        Alias {
          name: TextNode {
            value: "duplicate".into(),
            range: range((0, 6, 0, 15))
          },
          value: TextNode {
            value: "foo".into(),
            range: range((0, 19, 0, 22))
          },
          range: range((0, 0, 0, 22))
        },
        Alias {
          name: TextNode {
            value: "duplicate".into(),
            range: range((1, 6, 1, 15))
          },
          value: TextNode {
            value: "bar".into(),
            range: range((1, 19, 1, 22))
          },
          range: range((1, 0, 1, 22))
        }
      ]
    );
  }

  #[test]
  fn get_multiple_aliases() {
    let document = Document::from(indoc! {
      "
      alias a1 := foo
      alias a2 := bar
      "
    });

    let aliases = document.aliases();

    assert_eq!(aliases.len(), 2);

    assert_eq!(
      aliases,
      vec![
        Alias {
          name: TextNode {
            value: "a1".into(),
            range: range((0, 6, 0, 8)),
          },
          value: TextNode {
            value: "foo".into(),
            range: range((0, 12, 0, 15)),
          },
          range: range((0, 0, 0, 15)),
        },
        Alias {
          name: TextNode {
            value: "a2".into(),
            range: range((1, 6, 1, 8)),
          },
          value: TextNode {
            value: "bar".into(),
            range: range((1, 12, 1, 15)),
          },
          range: range((1, 0, 1, 15)),
        }
      ]
    );
  }

  #[test]
  fn get_multiple_settings() {
    let document = Document::from(indoc! {
      "
      set export := true
      set shell := ['foo']
      set bar := 'wow!'
      "
    });

    let settings = document.settings();

    assert_eq!(settings.len(), 3);

    assert_eq!(
      settings,
      vec![
        Setting {
          name: "export".into(),
          kind: SettingKind::Boolean(true),
          range: range((0, 0, 1, 0)),
        },
        Setting {
          name: "shell".into(),
          kind: SettingKind::Array,
          range: range((1, 0, 2, 0)),
        },
        Setting {
          name: "bar".into(),
          kind: SettingKind::String,
          range: range((2, 0, 3, 0)),
        }
      ]
    );
  }

  #[test]
  fn get_string_setting() {
    let document = Document::from(indoc! {
      "
      set bar := 'wow!'
      "
    });

    let settings = document.settings();

    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings,
      vec![Setting {
        name: "bar".into(),
        kind: SettingKind::String,
        range: range((0, 0, 1, 0)),
      }]
    );
  }

  #[test]
  fn get_variables() {
    let document = Document::from(indoc! {
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
      document.variables(),
      vec![
        Variable {
          name: TextNode {
            value: "tmpdir".into(),
            range: range((0, 0, 0, 6)),
          },
          export: false,
          unexport: false,
          content: "tmpdir  := `mktemp -d`".into(),
          range: range((0, 0, 1, 0)),
        },
        Variable {
          name: TextNode {
            value: "version".into(),
            range: range((1, 0, 1, 7)),
          },
          export: false,
          unexport: false,
          content: "version := \"0.2.7\"".into(),
          range: range((1, 0, 2, 0)),
        },
        Variable {
          name: TextNode {
            value: "tardir".into(),
            range: range((2, 0, 2, 6)),
          },
          export: false,
          unexport: false,
          content: "tardir  := tmpdir / \"awesomesauce-\" + version".into(),
          range: range((2, 0, 3, 0)),
        },
        Variable {
          name: TextNode {
            value: "tarball".into(),
            range: range((3, 0, 3, 7)),
          },
          export: false,
          unexport: false,
          content: "tarball := tardir + \".tar.gz\"".into(),
          range: range((3, 0, 4, 0)),
        },
        Variable {
          name: TextNode {
            value: "config".into(),
            range: range((4, 0, 4, 6)),
          },
          export: false,
          unexport: false,
          content: "config  := quote(config_dir() / \".project-config\")"
            .into(),
          range: range((4, 0, 5, 0)),
        },
        Variable {
          name: TextNode {
            value: "EDITOR".into(),
            range: range((5, 7, 5, 13)),
          },
          export: true,
          unexport: false,
          content: "EDITOR := 'nvim'".into(),
          range: range((5, 7, 6, 0)),
        },
      ]
    );
  }

  #[test]
  fn private_exported_variable_is_marked_exported() {
    let document = Document::from(indoc! {
      "
      [private]
      export PATH := '/usr/local/bin'
      "
    });

    let variables = document.variables();

    assert_eq!(variables.len(), 1);

    assert_eq!(
      variables,
      vec![Variable {
        name: TextNode {
          value: "PATH".into(),
          range: range((1, 7, 1, 11)),
        },
        export: true,
        unexport: false,
        content: "PATH := '/usr/local/bin'".into(),
        range: range((1, 7, 2, 0)),
      }]
    );
  }

  #[test]
  fn unexport_variable_is_marked_unexported() {
    let document = Document::from(indoc! {
      "
      unexport FOO := 'bar'
      "
    });

    let variables = document.variables();

    assert_eq!(variables.len(), 1);

    assert_eq!(
      variables,
      vec![Variable {
        name: TextNode {
          value: "FOO".into(),
          range: range((0, 9, 0, 12)),
        },
        export: false,
        unexport: true,
        content: "FOO := 'bar'".into(),
        range: range((0, 9, 1, 0)),
      }]
    );
  }

  #[test]
  fn multiple_recipes() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      bar:
        echo \"bar\"
      "
    });

    assert_eq!(
      document.find_recipe("foo"),
      Some(Recipe {
        name: TextNode {
          value: "foo".into(),
          range: range((0, 0, 0, 3))
        },
        attributes: vec![],
        dependencies: vec![],
        parameters: vec![],
        content: "foo:\n  echo \"foo\"".into(),
        range: range((0, 0, 3, 0)),
        shebang: None,
      })
    );

    assert_eq!(
      document.find_recipe("bar"),
      Some(Recipe {
        name: TextNode {
          value: "bar".into(),
          range: range((3, 0, 3, 3))
        },
        attributes: vec![],
        dependencies: vec![],
        parameters: vec![],
        content: "bar:\n  echo \"bar\"".into(),
        range: range((3, 0, 5, 0)),
        shebang: None,
      })
    );
  }

  #[test]
  fn node_at_position() {
    let document = Document::from(indoc! {"
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
    "});

    let node = document
      .node_at_position(lsp::Position {
        line: 1,
        character: 1,
      })
      .unwrap();

    assert_eq!(node.kind(), "recipe");
    assert_eq!(document.get_node_text(&node), "foo:\n  echo \"foo\"\n\n");

    let node = document
      .node_at_position(lsp::Position {
        line: 4,
        character: 6,
      })
      .unwrap();

    assert_eq!(node.kind(), "text");
    assert_eq!(document.get_node_text(&node), "echo \"bar\"");
  }

  #[test]
  fn node_at_position_handles_utf16_columns() {
    let document = Document::from(indoc! {"
      foo:
        echo \"a🧪b\"
    "});

    let node = document
      .node_at_position(lsp::Position {
        line: 1,
        character: 11,
      })
      .unwrap();

    assert_eq!(node.kind(), "text");
    assert_eq!(document.get_node_text(&node), "echo \"a🧪b\"");
  }

  #[test]
  fn recipe_with_default_parameter() {
    let document = Document::from(indoc! {
      "
      baz first second=\"default\":
        echo \"{{first}} {{second}}\"
      "
    });

    assert_eq!(
      document.find_recipe("baz"),
      Some(Recipe {
        name: TextNode {
          value: "baz".into(),
          range: range((0, 0, 0, 3))
        },
        attributes: vec![],
        dependencies: vec![],
        parameters: vec![
          Parameter {
            name: "first".into(),
            kind: ParameterKind::Normal,
            default_value: None,
            content: "first".into(),
            range: range((0, 4, 0, 9)),
          },
          Parameter {
            name: "second".into(),
            kind: ParameterKind::Normal,
            default_value: Some("\"default\"".into()),
            content: "second=\"default\"".into(),
            range: range((0, 10, 0, 26)),
          }
        ],
        content:
          "baz first second=\"default\":\n  echo \"{{first}} {{second}}\""
            .into(),
        range: range((0, 0, 2, 0)),
        shebang: None,
      })
    );
  }

  #[test]
  fn recipe_with_dependency() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
      "
    });

    assert_eq!(
      document.find_recipe("bar"),
      Some(Recipe {
        name: TextNode {
          value: "bar".into(),
          range: range((3, 0, 3, 3))
        },
        attributes: vec![],
        dependencies: vec![Dependency {
          name: "foo".into(),
          arguments: vec![],
          range: range((3, 5, 3, 8)),
        }],
        parameters: vec![],
        content: "bar: foo\n  echo \"bar\"".into(),
        range: range((3, 0, 5, 0)),
        shebang: None,
      })
    );
  }

  #[test]
  fn recipe_with_module_path_dependency() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      bar:
        echo \"bar\"

      baz: tools::foo
        echo \"baz\"
      "
    });

    assert_eq!(
      document.find_recipe("baz"),
      Some(Recipe {
        name: TextNode {
          value: "baz".into(),
          range: range((6, 0, 6, 3))
        },
        attributes: vec![],
        dependencies: vec![Dependency {
          name: "tools::foo".into(),
          arguments: vec![],
          range: range((6, 5, 6, 15)),
        }],
        parameters: vec![],
        content: "baz: tools::foo\n  echo \"baz\"".into(),
        range: range((6, 0, 8, 0)),
        shebang: None,
      })
    );
  }

  #[test]
  fn recipe_with_dependency_arguments() {
    let document = Document::from(indoc! {
      "
      foo arg1 arg2:
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo 'value1' 'value2')
        echo \"bar\"
      "
    });

    assert_eq!(
      document.find_recipe("bar"),
      Some(Recipe {
        name: TextNode {
          value: "bar".into(),
          range: range((3, 0, 3, 3))
        },
        attributes: vec![],
        dependencies: vec![Dependency {
          name: "foo".into(),
          arguments: vec![
            TextNode {
              value: "'value1'".into(),
              range: range((3, 10, 3, 18)),
            },
            TextNode {
              value: "'value2'".into(),
              range: range((3, 19, 3, 27)),
            }
          ],
          range: range((3, 5, 3, 28)),
        }],
        parameters: vec![],
        content: "bar: (foo 'value1' 'value2')\n  echo \"bar\"".into(),
        range: range((3, 0, 5, 0)),
        shebang: None,
      })
    );
  }

  #[test]
  fn recipe_with_shebang() {
    let document = Document::from(indoc! {
      "
      foo:
        #!/usr/bin/env bash
        echo \"foo\"
      "
    });

    let recipe = document.find_recipe("foo").unwrap();

    assert_eq!(
      recipe.shebang,
      Some(TextNode {
        value: "#!/usr/bin/env bash".into(),
        range: range((1, 2, 1, 21)),
      })
    );
  }

  #[test]
  fn recipe_with_multiple_dependencies() {
    let document = Document::from(indoc! {
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
      document.find_recipe("baz"),
      Some(Recipe {
        name: TextNode {
          value: "baz".into(),
          range: range((6, 0, 6, 3))
        },
        attributes: vec![],
        dependencies: vec![
          Dependency {
            name: "foo".into(),
            arguments: vec![],
            range: range((6, 5, 6, 8)),
          },
          Dependency {
            name: "bar".into(),
            arguments: vec![],
            range: range((6, 9, 6, 12)),
          }
        ],
        parameters: vec![],
        content: "baz: foo bar\n  echo \"baz\"".into(),
        range: range((6, 0, 8, 0)),
        shebang: None,
      })
    );
  }

  #[test]
  fn recipe_with_parameters() {
    let document = Document::from(indoc! {
      "
      bar target $lol:
        echo \"Building {{target}}\"
      "
    });

    assert_eq!(
      document.find_recipe("bar"),
      Some(Recipe {
        name: TextNode {
          value: "bar".into(),
          range: range((0, 0, 0, 3))
        },
        attributes: vec![],
        dependencies: vec![],
        parameters: vec![
          Parameter {
            name: "target".into(),
            kind: ParameterKind::Normal,
            default_value: None,
            content: "target".into(),
            range: range((0, 4, 0, 10)),
          },
          Parameter {
            name: "lol".into(),
            kind: ParameterKind::Export,
            default_value: None,
            content: "$lol".into(),
            range: range((0, 11, 0, 15)),
          }
        ],
        content: "bar target $lol:\n  echo \"Building {{target}}\"".into(),
        range: range((0, 0, 2, 0)),
        shebang: None,
      })
    );
  }

  #[test]
  fn recipe_with_variadic_parameter() {
    let document = Document::from(indoc! {
      "
      baz first +second=\"default\":
        echo \"{{first}} {{second}}\"
      "
    });

    assert_eq!(
      document.find_recipe("baz"),
      Some(Recipe {
        name: TextNode {
          value: "baz".into(),
          range: range((0, 0, 0, 3))
        },
        attributes: vec![],
        dependencies: vec![],
        parameters: vec![
          Parameter {
            name: "first".into(),
            kind: ParameterKind::Normal,
            default_value: None,
            content: "first".into(),
            range: range((0, 4, 0, 9)),
          },
          Parameter {
            name: "second".into(),
            kind: ParameterKind::Variadic(VariadicType::OneOrMore),
            default_value: Some("\"default\"".into()),
            content: "+second=\"default\"".into(),
            range: range((0, 10, 0, 27)),
          }
        ],
        content:
          "baz first +second=\"default\":\n  echo \"{{first}} {{second}}\""
            .into(),
        range: range((0, 0, 2, 0)),
        shebang: None,
      })
    );
  }

  #[test]
  fn recipe_without_parameters_or_dependencies() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    assert_eq!(
      document.find_recipe("foo"),
      Some(Recipe {
        name: TextNode {
          value: "foo".into(),
          range: range((0, 0, 0, 3))
        },
        attributes: vec![],
        dependencies: vec![],
        parameters: vec![],
        content: "foo:\n  echo \"foo\"".into(),
        range: range((0, 0, 2, 0)),
        shebang: None,
      })
    );
  }

  #[test]
  fn recipe_with_attributes() {
    let document = Document::from(indoc! {
      "
      [private]
      [description: \"This is a test recipe\"]
      [tags(\"test\", \"example\")]
      foo:
        echo \"foo\"
      "
    });

    let recipe = document.find_recipe("foo").unwrap();

    assert_eq!(recipe.attributes.len(), 3);

    assert_eq!(
      recipe.attributes,
      vec![
        Attribute {
          name: TextNode {
            value: "private".into(),
            range: range((0, 1, 0, 8)),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: range((0, 0, 1, 0)),
        },
        Attribute {
          name: TextNode {
            value: "description".into(),
            range: range((1, 1, 1, 12)),
          },
          arguments: vec![TextNode {
            value: "\"This is a test recipe\"".into(),
            range: range((1, 14, 1, 37)),
          }],
          target: Some(AttributeTarget::Recipe),
          range: range((1, 0, 2, 0)),
        },
        Attribute {
          name: TextNode {
            value: "tags".into(),
            range: range((2, 1, 2, 5)),
          },
          arguments: vec![
            TextNode {
              value: "\"test\"".into(),
              range: range((2, 6, 2, 12)),
            },
            TextNode {
              value: "\"example\"".into(),
              range: range((2, 14, 2, 23)),
            }
          ],
          target: Some(AttributeTarget::Recipe),
          range: range((2, 0, 3, 0)),
        }
      ]
    );
  }

  #[test]
  fn list_document_attributes() {
    let document = Document::from(indoc! {
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

    let attributes = document.attributes();

    assert_eq!(
      attributes,
      vec![
        Attribute {
          arguments: vec![],
          name: TextNode {
            value: "private".into(),
            range: range((0, 1, 0, 8)),
          },
          range: range((0, 0, 1, 0)),
          target: Some(AttributeTarget::Recipe),
        },
        Attribute {
          arguments: vec![TextNode {
            value: "\"desc\"".into(),
            range: range((0, 23, 0, 29)),
          }],
          name: TextNode {
            value: "description".into(),
            range: range((0, 10, 0, 21)),
          },
          range: range((0, 0, 1, 0)),
          target: Some(AttributeTarget::Recipe),
        },
        Attribute {
          arguments: vec![],
          name: TextNode {
            value: "alias_attr".into(),
            range: range((4, 1, 4, 11)),
          },
          range: range((4, 0, 5, 0)),
          target: Some(AttributeTarget::Alias),
        },
        Attribute {
          arguments: vec![TextNode {
            value: "\"value\"".into(),
            range: range((7, 10, 7, 17)),
          }],
          name: TextNode {
            value: "var_attr".into(),
            range: range((7, 1, 7, 9)),
          },
          range: range((7, 0, 8, 0)),
          target: Some(AttributeTarget::Assignment),
        },
        Attribute {
          arguments: vec![],
          name: TextNode {
            value: "export_attr".into(),
            range: range((10, 1, 10, 12)),
          },
          range: range((10, 0, 11, 0)),
          target: Some(AttributeTarget::Assignment),
        },
        Attribute {
          arguments: vec![],
          name: TextNode {
            value: "module_attr".into(),
            range: range((13, 1, 13, 12)),
          },
          range: range((13, 0, 14, 0)),
          target: Some(AttributeTarget::Module),
        },
      ],
    );
  }

  #[test]
  fn list_function_calls() {
    let document = Document::from(indoc! {"
      foo:
        echo {{arch()}}
        echo {{env_var(\"HOME\", \"fallback\")}}
    "});

    let calls = document.function_calls();

    assert_eq!(
      calls,
      vec![
        FunctionCall {
          arguments: vec![],
          name: TextNode {
            value: "arch".into(),
            range: range((1, 9, 1, 13)),
          },
          range: range((1, 9, 1, 15)),
        },
        FunctionCall {
          arguments: vec![
            TextNode {
              value: "\"HOME\"".into(),
              range: range((2, 17, 2, 23)),
            },
            TextNode {
              value: "\"fallback\"".into(),
              range: range((2, 25, 2, 35)),
            },
          ],
          name: TextNode {
            value: "env_var".into(),
            range: range((2, 9, 2, 16)),
          },
          range: range((2, 9, 2, 36)),
        },
      ],
    );
  }
}
