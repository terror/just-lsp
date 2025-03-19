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

    document.parse()?;

    Ok(document)
  }
}

impl Document {
  pub(crate) fn apply_change(
    &mut self,
    params: lsp::DidChangeTextDocumentParams,
  ) -> Result {
    let lsp::DidChangeTextDocumentParams {
      content_changes, ..
    } = params;

    let edits = content_changes
      .iter()
      .map(|change| self.content.build_edit(change))
      .collect::<Result<Vec<_>, _>>()?;

    edits.iter().for_each(|edit| self.content.apply_edit(edit));

    self.parse()?;

    Ok(())
  }

  pub(crate) fn find_child_by_kind<'a>(
    &'a self,
    node: &'a Node,
    kind: &str,
  ) -> Option<Node<'a>> {
    (0..node.child_count())
      .filter_map(|i| node.child(i))
      .find(|child| child.kind() == kind)
  }

  pub(crate) fn find_child_by_kind_at_position<'a>(
    &'a self,
    node: &'a Node,
    kind: &str,
    position: usize,
  ) -> Option<Node<'a>> {
    node.child(position).filter(|child| child.kind() == kind)
  }

  pub(crate) fn find_nodes_by_kind(&self, kind: &str) -> Vec<Node> {
    let mut nodes = Vec::new();

    if let Some(tree) = &self.tree {
      let mut cursor = tree.root_node().walk();
      Self::collect_nodes(&mut cursor, kind, &mut nodes);
    }

    nodes
  }

  pub(crate) fn find_recipe(&self, name: &str) -> Option<Recipe> {
    self
      .get_recipes()
      .into_iter()
      .find(|recipe| recipe.name == name)
  }

  pub(crate) fn find_references(&self, name: &str) -> Vec<lsp::Location> {
    self
      .find_nodes_by_kind("identifier")
      .into_iter()
      .filter(|identifier| self.get_node_text(identifier) == name)
      .map(|identifier| lsp::Location {
        uri: self.uri.clone(),
        range: identifier.get_range(),
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
    self
      .find_nodes_by_kind("recipe")
      .iter()
      .filter_map(|recipe_node| {
        let header = self.find_child_by_kind(recipe_node, "recipe_header")?;

        let name_node = self.find_child_by_kind(&header, "identifier")?;

        let name = self.get_node_text(&name_node);

        let dependencies = self
          .find_child_by_kind(&header, "dependencies")
          .map(|deps_node| {
            (0..deps_node.named_child_count())
              .filter_map(|i| deps_node.named_child(i))
              .filter(|child| child.kind() == "dependency")
              .filter_map(|dep_node| {
                self
                  .find_child_by_kind(&dep_node, "identifier")
                  .map(|id_node| self.get_node_text(&id_node))
              })
              .collect::<Vec<_>>()
          })
          .unwrap_or_default();

        Some(Recipe {
          name,
          dependencies,
          content: self.get_node_text(recipe_node).trim().to_string(),
          range: recipe_node.get_range(),
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

  pub(crate) fn parse(&mut self) -> Result {
    let mut parser = Parser::new();

    let language = unsafe { tree_sitter_just() };

    parser.set_language(&language)?;

    self.tree = parser.parse(self.content.to_string(), None);

    Ok(())
  }

  fn collect_nodes<'a>(
    cursor: &mut TreeCursor<'a>,
    kind: &str,
    nodes: &mut Vec<Node<'a>>,
  ) {
    let node = cursor.node();

    if node.kind() == kind {
      nodes.push(node);
    }

    if cursor.goto_first_child() {
      loop {
        Self::collect_nodes(cursor, kind, nodes);

        if !cursor.goto_next_sibling() {
          break;
        }
      }

      cursor.goto_parent();
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

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
  fn find_nodes_by_kind() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar:
        echo \"bar\"

      baz:
        echo \"baz\"
    "});

    let recipes = doc.find_nodes_by_kind("recipe");
    assert_eq!(recipes.len(), 3);

    let identifiers = doc.find_nodes_by_kind("identifier");
    assert_eq!(identifiers.len(), 3);
  }

  #[test]
  fn get_recipes() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"

      baz: foo bar
        echo \"baz\"
      "
    });

    assert_eq!(
      doc.get_recipes(),
      vec![
        Recipe {
          name: "foo".into(),
          dependencies: vec![],
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
        },
        Recipe {
          name: "bar".into(),
          dependencies: vec!["foo".into()],
          content: "bar: foo\n  echo \"bar\"".into(),
          range: lsp::Range {
            start: lsp::Position {
              line: 3,
              character: 0
            },
            end: lsp::Position {
              line: 6,
              character: 0
            },
          }
        },
        Recipe {
          name: "baz".into(),
          dependencies: vec!["foo".into(), "bar".into()],
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
        }
      ]
    );
  }

  #[test]
  fn find_child_by_kind() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"
    "});

    let recipes = doc.find_nodes_by_kind("recipe");
    assert_eq!(recipes.len(), 1);

    let recipe = &recipes[0];

    let header = doc.find_child_by_kind(recipe, "recipe_header");
    assert!(header.is_some());

    let body = doc.find_child_by_kind(recipe, "recipe_body");
    assert!(body.is_some());

    let nonexistent = doc.find_child_by_kind(recipe, "nonexistent");
    assert!(nonexistent.is_none());
  }

  #[test]
  fn find_references() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"

      alias baz := foo
      "
    });

    let references = doc.find_references("foo");

    assert_eq!(references.len(), 3);
    assert_eq!(references[0].range.start.line, 0);
    assert_eq!(references[1].range.start.line, 3);
    assert_eq!(references[2].range.start.line, 6);
  }

  #[test]
  fn find_child_by_kind_at_position() {
    let doc = document(indoc! {
      "
      alias foo := bar
      "
    });

    let alias_nodes = doc.find_nodes_by_kind("alias");

    assert_eq!(alias_nodes.len(), 2);

    let alias_node = alias_nodes.first().unwrap();

    let alias_name =
      doc.find_child_by_kind_at_position(alias_node, "identifier", 1);

    assert!(alias_name.is_some());
    assert_eq!(doc.get_node_text(&alias_name.unwrap()), "foo");

    let target_name =
      doc.find_child_by_kind_at_position(alias_node, "identifier", 3);

    assert!(target_name.is_some());
    assert_eq!(doc.get_node_text(&target_name.unwrap()), "bar");
  }
}
