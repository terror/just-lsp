use super::*;

#[derive(Debug)]
pub struct Document {
  content: Rope,
  tree: Option<Tree>,
  uri: lsp::Url,
}

impl From<lsp::DidOpenTextDocumentParams> for Document {
  fn from(value: lsp::DidOpenTextDocumentParams) -> Self {
    let document = value.text_document;

    let mut doc = Self {
      content: Rope::from_str(&document.text),
      tree: None,
      uri: document.uri,
    };

    doc.parse();

    doc
  }
}

impl Document {
  pub(crate) fn apply_change(
    &mut self,
    params: lsp::DidChangeTextDocumentParams,
  ) -> Result {
    let edits = params
      .content_changes
      .iter()
      .map(|change| self.content.build_edit(change))
      .collect::<Result<Vec<_>, _>>()?;

    for edit in edits {
      self.content.apply_edit(&edit);
    }

    self.parse();

    Ok(())
  }

  pub(crate) fn collect_diagnostics(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(self.validate_aliases());
    diagnostics.extend(self.validate_dependencies());
    diagnostics
  }

  pub(crate) fn find_recipe_by_name<'a>(
    &'a self,
    name: &str,
  ) -> Option<Node<'a>> {
    self
      .find_nodes_by_kind("recipe")
      .into_iter()
      .find(|recipe_node| {
        self
          .find_child_by_kind(recipe_node, "recipe_header")
          .as_ref()
          .and_then(|header| self.find_child_by_kind(&header, "identifier"))
          .map(|identifier| self.get_node_text(&identifier) == name)
          .unwrap_or(false)
      })
  }

  pub(crate) fn find_recipe_references(
    &self,
    recipe_name: &str,
  ) -> Vec<lsp::Location> {
    let mut locations = Vec::new();

    let dependency_nodes = self.find_nodes_by_kind("dependency");

    for dependency_node in dependency_nodes {
      if let Some(identifier) =
        self.find_child_by_kind(&dependency_node, "identifier")
      {
        let dep_name = self.get_node_text(&identifier);

        if dep_name == recipe_name {
          locations.push(lsp::Location {
            uri: self.uri.clone(),
            range: self.node_to_range(&identifier),
          });
        }
      }
    }

    let alias_nodes = self.find_nodes_by_kind("alias");

    for alias_node in alias_nodes {
      if let Some(identifier) =
        self.find_child_by_kind_at_position(&alias_node, "identifier", 3)
      {
        let alias_target = self.get_node_text(&identifier);

        if alias_target == recipe_name {
          locations.push(lsp::Location {
            uri: self.uri.clone(),
            range: self.node_to_range(&identifier),
          });
        }
      }
    }

    locations
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

  pub(crate) fn node_at_position(
    &self,
    position: lsp::Position,
  ) -> Option<Node> {
    if let Some(tree) = &self.tree {
      let point = self.position_to_point(position);
      Some(tree.root_node().descendant_for_point_range(point, point)?)
    } else {
      None
    }
  }

  pub(crate) fn parse(&mut self) {
    let mut parser = Parser::new();

    unsafe {
      parser
        .set_language(&tree_sitter_just())
        .expect("Failed to load `tree-sitter-just`");
    }

    let text = self.content.to_string();

    self.tree = parser.parse(&text, None);
  }

  fn position_to_point(&self, position: lsp::Position) -> Point {
    Point {
      row: position.line as usize,
      column: position.character as usize,
    }
  }

  fn point_to_position(&self, point: Point) -> lsp::Position {
    lsp::Position {
      line: point.row as u32,
      character: point.column as u32,
    }
  }

  pub(crate) fn node_to_range(&self, node: &Node) -> lsp::Range {
    lsp::Range {
      start: self.point_to_position(node.start_position()),
      end: self.point_to_position(node.end_position()),
    }
  }

  fn find_nodes_by_kind(&self, kind: &str) -> Vec<Node> {
    let mut nodes = Vec::new();

    if let Some(tree) = &self.tree {
      let mut cursor = tree.root_node().walk();
      self.collect_nodes(&mut cursor, kind, &mut nodes);
    }

    nodes
  }

  fn collect_nodes<'a>(
    &self,
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
        self.collect_nodes(cursor, kind, nodes);

        if !cursor.goto_next_sibling() {
          break;
        }
      }

      cursor.goto_parent();
    }
  }

  fn find_child_by_kind<'a>(
    &'a self,
    node: &'a Node,
    kind: &str,
  ) -> Option<Node<'a>> {
    (0..node.child_count())
      .filter_map(|i| node.child(i))
      .find(|child| child.kind() == kind)
  }

  fn find_child_by_kind_at_position<'a>(
    &'a self,
    node: &'a Node,
    kind: &str,
    position: usize,
  ) -> Option<Node<'a>> {
    node.child(position).filter(|child| child.kind() == kind)
  }

  fn get_recipe_names(&self) -> Vec<String> {
    self
      .find_nodes_by_kind("recipe")
      .iter()
      .filter_map(|recipe| {
        self
          .find_child_by_kind(recipe, "recipe_header")
          .and_then(|header| {
            (0..header.named_child_count())
              .filter_map(|i| header.named_child(i))
              .find(|child| child.kind() == "identifier")
              .map(|identifier| self.get_node_text(&identifier))
          })
      })
      .collect()
  }

  fn validate_aliases(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let recipe_names = self.get_recipe_names();

    let alias_nodes = self.find_nodes_by_kind("alias");

    for alias_node in alias_nodes {
      if let Some(identifier) =
        self.find_child_by_kind_at_position(&alias_node, "identifier", 3)
      {
        let target_name = self.get_node_text(&identifier);

        if !recipe_names.contains(&target_name) {
          diagnostics.push(lsp::Diagnostic {
            range: self.node_to_range(&alias_node),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!("Recipe '{}' not found", target_name),
            ..Default::default()
          });
        }
      }
    }

    diagnostics
  }

  fn validate_dependencies(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let recipe_names = self.get_recipe_names();

    for recipe_node in self.find_nodes_by_kind("recipe") {
      if let Some(header) =
        self.find_child_by_kind(&recipe_node, "recipe_header")
      {
        if let Some(dependencies) =
          self.find_child_by_kind(&header, "dependencies")
        {
          for i in 0..dependencies.named_child_count() {
            if let Some(dependency) = dependencies.named_child(i) {
              if dependency.kind() == "dependency" {
                if let Some(identifier) =
                  self.find_child_by_kind(&dependency, "identifier")
                {
                  let dependency_name = self.get_node_text(&identifier);

                  if !recipe_names.contains(&dependency_name) {
                    diagnostics.push(lsp::Diagnostic {
                      range: self.node_to_range(&identifier),
                      severity: Some(lsp::DiagnosticSeverity::ERROR),
                      source: Some("just-lsp".to_string()),
                      message: format!(
                        "Recipe '{}' not found",
                        dependency_name
                      ),
                      ..Default::default()
                    });
                  }
                }
              }
            }
          }
        }
      }
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  fn document(content: &str) -> Document {
    Document::from(lsp::DidOpenTextDocumentParams {
      text_document: lsp::TextDocumentItem {
        uri: lsp::Url::parse("file:///test.just").unwrap(),
        language_id: "just".to_string(),
        version: 1,
        text: content.to_string(),
      },
    })
  }

  fn position(line: u32, character: u32) -> lsp::Position {
    lsp::Position { line, character }
  }

  #[test]
  fn document_creation() {
    let content = indoc! {"
      foo:
        echo foo
    "};

    let doc = document(content);

    assert_eq!(doc.content.to_string(), content);

    assert!(doc.tree.is_some());
  }

  #[test]
  fn find_recipe_by_name() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar:
        echo \"bar\"
    "});

    let recipe = doc.find_recipe_by_name("foo").unwrap();
    assert!(doc.get_node_text(&recipe).contains("foo"));

    let recipe = doc.find_recipe_by_name("bar").unwrap();
    assert!(doc.get_node_text(&recipe).contains("bar"));

    assert!(doc.find_recipe_by_name("baz").is_none());
  }

  #[test]
  fn node_at_position() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
    "});

    let node = doc.node_at_position(position(1, 1)).unwrap();
    assert_eq!(node.kind(), "recipe");
    assert_eq!(doc.get_node_text(&node), "foo:\n  echo \"foo\"\n\n");

    let node = doc.node_at_position(position(4, 6)).unwrap();
    assert_eq!(node.kind(), "text");
    assert_eq!(doc.get_node_text(&node), "echo \"bar\"");
  }

  #[test]
  fn validate_dependencies() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar: baz
        echo \"bar\"
    "});

    let diagnostics = doc.validate_dependencies();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Recipe 'baz' not found");
    assert_eq!(diagnostics[0].range.start.line, 3);

    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
    "});

    let diagnostics = doc.validate_dependencies();
    assert_eq!(diagnostics.len(), 0);

    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar: missing1 missing2
        echo \"bar\"
    "});

    let diagnostics = doc.validate_dependencies();
    assert_eq!(diagnostics.len(), 2);
  }

  #[test]
  fn validate_aliases() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      alias bar := baz
    "});

    let diagnostics = doc.validate_aliases();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Recipe 'baz' not found");

    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      alias bar := foo
    "});

    let diagnostics = doc.validate_aliases();
    assert_eq!(diagnostics.len(), 0);
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
  fn get_recipe_names() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar:
        echo \"bar\"

      baz:
        echo \"baz\"
    "});

    let names = doc.get_recipe_names();
    assert_eq!(names.len(), 3);
    assert!(names.contains(&"foo".to_string()));
    assert!(names.contains(&"bar".to_string()));
    assert!(names.contains(&"baz".to_string()));
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
          start: position(1, 7),
          end: position(1, 13),
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
  fn find_recipe_references() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"

      alias baz := foo
      "
    });

    let references = doc.find_recipe_references("foo");

    assert_eq!(references.len(), 2);
    assert_eq!(references[0].range.start.line, 3);
    assert_eq!(references[1].range.start.line, 6);
  }

  #[test]
  fn collect_diagnostics() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar: missing
        echo \"bar\"

      alias baz := nonexistent
      "
    });

    let diagnostics = doc.collect_diagnostics();

    assert_eq!(diagnostics.len(), 2);

    let messages: Vec<String> =
      diagnostics.iter().map(|d| d.message.clone()).collect();

    assert!(messages.contains(&"Recipe 'missing' not found".to_string()));
    assert!(messages.contains(&"Recipe 'nonexistent' not found".to_string()));
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
      doc.find_child_by_kind_at_position(&alias_node, "identifier", 1);

    assert!(alias_name.is_some());
    assert_eq!(doc.get_node_text(&alias_name.unwrap()), "foo");

    let target_name =
      doc.find_child_by_kind_at_position(&alias_node, "identifier", 3);

    assert!(target_name.is_some());
    assert_eq!(doc.get_node_text(&target_name.unwrap()), "bar");
  }
}
