use super::*;

#[derive(Debug)]
pub struct Document {
  content: Rope,
  tree: Option<Tree>,
  uri: lsp::Url,
  version: i32,
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
    let edits = params
      .content_changes
      .iter()
      .map(|change| self.content.build_edit(change))
      .collect::<Result<Vec<_>, _>>()?;

    edits.iter().for_each(|edit| self.content.apply_edit(edit));

    self.parse()?;

    Ok(())
  }

  pub(crate) fn collect_diagnostics(&self) -> Vec<lsp::Diagnostic> {
    self
      .collect_parser_errors()
      .into_iter()
      .chain(self.validate_aliases())
      .chain(self.validate_dependencies())
      .chain(self.validate_function_calls())
      .chain(self.validate_settings())
      .collect()
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
          .and_then(|header| self.find_child_by_kind(header, "identifier"))
          .map(|identifier| self.get_node_text(&identifier) == name)
          .unwrap_or(false)
      })
  }

  pub(crate) fn find_references(&self, name: &str) -> Vec<lsp::Location> {
    self
      .find_nodes_by_kind("identifier")
      .into_iter()
      .filter(|identifier| self.get_node_text(identifier) == name)
      .map(|identifier| lsp::Location {
        uri: self.uri.clone(),
        range: self.node_to_range(&identifier),
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

  pub(crate) fn node_to_range(&self, node: &Node) -> lsp::Range {
    lsp::Range {
      start: self.point_to_position(node.start_position()),
      end: self.point_to_position(node.end_position()),
    }
  }

  pub(crate) fn parse(&mut self) -> Result {
    let mut parser = Parser::new();

    let language = unsafe { tree_sitter_just() };

    parser.set_language(&language)?;

    self.tree = parser.parse(self.content.to_string(), None);

    Ok(())
  }

  pub(crate) fn version(&self) -> i32 {
    self.version
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

  fn collect_parser_errors(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    if let Some(tree) = &self.tree {
      let mut cursor = tree.root_node().walk();
      self.collect_parser_errors_rec(&mut cursor, &mut diagnostics);
    }

    diagnostics
  }

  fn collect_parser_errors_rec(
    &self,
    cursor: &mut TreeCursor<'_>,
    diagnostics: &mut Vec<lsp::Diagnostic>,
  ) {
    let node = cursor.node();

    if node.is_error() {
      diagnostics.push(lsp::Diagnostic {
        range: self.node_to_range(&node),
        severity: Some(lsp::DiagnosticSeverity::ERROR),
        source: Some("just-lsp".to_string()),
        message: "Syntax error".to_string(),
        ..Default::default()
      });
    }

    if node.is_missing() {
      diagnostics.push(lsp::Diagnostic {
        range: self.node_to_range(&node),
        severity: Some(lsp::DiagnosticSeverity::ERROR),
        source: Some("just-lsp".to_string()),
        message: "Missing syntax element".to_string(),
        ..Default::default()
      });
    }

    if cursor.goto_first_child() {
      loop {
        self.collect_parser_errors_rec(cursor, diagnostics);

        if !cursor.goto_next_sibling() {
          break;
        }
      }

      cursor.goto_parent();
    }
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

  fn find_child_by_kind_at_position<'a>(
    &'a self,
    node: &'a Node,
    kind: &str,
    position: usize,
  ) -> Option<Node<'a>> {
    node.child(position).filter(|child| child.kind() == kind)
  }

  fn find_nodes_by_kind(&self, kind: &str) -> Vec<Node> {
    let mut nodes = Vec::new();

    if let Some(tree) = &self.tree {
      let mut cursor = tree.root_node().walk();
      Self::collect_nodes(&mut cursor, kind, &mut nodes);
    }

    nodes
  }

  pub(crate) fn get_recipe_names(&self) -> Vec<String> {
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

  fn point_to_position(&self, point: Point) -> lsp::Position {
    lsp::Position {
      line: point.row as u32,
      character: point.column as u32,
    }
  }

  fn position_to_point(&self, position: lsp::Position) -> Point {
    Point {
      row: position.line as usize,
      column: position.character as usize,
    }
  }

  fn validate_aliases(&self) -> Vec<lsp::Diagnostic> {
    let recipe_names = self.get_recipe_names();

    let alias_nodes = self.find_nodes_by_kind("alias");

    alias_nodes
      .into_iter()
      .filter_map(|alias_node| {
        self
          .find_child_by_kind_at_position(&alias_node, "identifier", 3)
          .filter(|identifier| {
            !recipe_names.contains(&self.get_node_text(identifier))
          })
          .map(|identifier| lsp::Diagnostic {
            range: self.node_to_range(&alias_node),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("just-lsp".to_string()),
            message: format!(
              "Recipe '{}' not found",
              self.get_node_text(&identifier)
            ),
            related_information: None,
            tags: None,
            data: None,
          })
      })
      .collect()
  }

  fn validate_dependencies(&self) -> Vec<lsp::Diagnostic> {
    let recipe_names = self.get_recipe_names();

    let recipe_nodes = self.find_nodes_by_kind("recipe");

    recipe_nodes
      .iter()
      .flat_map(|recipe_node| {
        let recipe_header =
          self.find_child_by_kind(recipe_node, "recipe_header");

        let dependencies = recipe_header
          .as_ref()
          .and_then(|header| self.find_child_by_kind(header, "dependencies"));

        dependencies.map_or(Vec::new(), |dependencies| {
          (0..dependencies.named_child_count())
            .filter_map(|i| dependencies.named_child(i))
            .filter(|dependency| dependency.kind() == "dependency")
            .filter_map(|dependency| {
              let identifier =
                self.find_child_by_kind(&dependency, "identifier");

              identifier.map(|identifier| {
                let text = self.get_node_text(&identifier);

                (!recipe_names.contains(&text)).then_some(lsp::Diagnostic {
                  range: self.node_to_range(&identifier),
                  severity: Some(lsp::DiagnosticSeverity::ERROR),
                  source: Some("just-lsp".to_string()),
                  message: format!("Recipe '{}' not found", text),
                  ..Default::default()
                })
              })
            })
            .flatten()
            .collect::<Vec<_>>()
        })
      })
      .collect()
  }

  fn validate_function_calls(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let function_calls = self.find_nodes_by_kind("function_call");

    for function_call in function_calls {
      if let Some(name_node) =
        self.find_child_by_kind(&function_call, "identifier")
      {
        let function_name = self.get_node_text(&name_node);

        if let Some(function) = constants::FUNCTIONS
          .iter()
          .find(|f| f.name == function_name)
        {
          let arguments = self.find_child_by_kind(&function_call, "sequence");

          let arg_count = arguments.map_or(0, |args| args.named_child_count());

          if arg_count < function.required_args {
            diagnostics.push(lsp::Diagnostic {
              range: self.node_to_range(&function_call),
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              source: Some("just-lsp".to_string()),
              message: format!(
              "Function '{}' requires at least {} argument(s), but {} provided",
              function_name, function.required_args, arg_count
            ),
              ..Default::default()
            });
          } else if !function.accepts_variadic
            && arg_count > function.required_args
          {
            diagnostics.push(lsp::Diagnostic {
              range: self.node_to_range(&function_call),
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              source: Some("just-lsp".to_string()),
              message: format!(
                "Function '{}' accepts {} argument(s), but {} provided",
                function_name, function.required_args, arg_count
              ),
              ..Default::default()
            });
          }
        } else {
          diagnostics.push(lsp::Diagnostic {
            range: self.node_to_range(&name_node),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!("Unknown function '{}'", function_name),
            ..Default::default()
          });
        }
      }
    }

    diagnostics
  }

  fn validate_settings(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let setting_nodes = self.find_nodes_by_kind("setting");

    for setting_node in setting_nodes.clone() {
      if let Some(name_node) =
        self.find_child_by_kind(&setting_node, "identifier")
      {
        let setting_name = self.get_node_text(&name_node);

        let valid_setting = constants::SETTINGS
          .iter()
          .find(|setting| setting.name == setting_name);

        if let Some(setting) = valid_setting {
          if let Some(setting_value_node) = setting_node.child(3) {
            let (setting_value_kind, setting_value) = (
              setting_value_node.kind(),
              self.get_node_text(&setting_value_node),
            );

            match setting.kind {
              SettingKind::Boolean => {
                if setting_value_kind != "boolean" {
                  diagnostics.push(lsp::Diagnostic {
                    range: self.node_to_range(&setting_value_node),
                    severity: Some(lsp::DiagnosticSeverity::ERROR),
                    source: Some("just-lsp".to_string()),
                    message: format!(
                      "Setting '{}' expects a boolean value",
                      setting_name
                    ),
                    ..Default::default()
                  });
                }
              }
              SettingKind::String => {
                if setting_value_kind != "string" {
                  diagnostics.push(lsp::Diagnostic {
                    range: self.node_to_range(&setting_value_node),
                    severity: Some(lsp::DiagnosticSeverity::ERROR),
                    source: Some("just-lsp".to_string()),
                    message: format!(
                      "Setting '{}' expects a string value",
                      setting_name
                    ),
                    ..Default::default()
                  });
                }
              }
              SettingKind::Array => {
                if !setting_value.starts_with('[')
                  || !setting_value.ends_with(']')
                {
                  diagnostics.push(lsp::Diagnostic {
                    range: self.node_to_range(&setting_value_node),
                    severity: Some(lsp::DiagnosticSeverity::ERROR),
                    source: Some("just-lsp".to_string()),
                    message: format!(
                      "Setting '{}' expects an array value",
                      setting_name
                    ),
                    ..Default::default()
                  });
                }
              }
            }
          }
        } else {
          diagnostics.push(lsp::Diagnostic {
            range: self.node_to_range(&name_node),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!("Unknown setting '{}'", setting_name),
            ..Default::default()
          });
        }
      }
    }

    let mut seen = std::collections::HashSet::new();

    for setting_node in setting_nodes {
      if let Some(name_node) =
        self.find_child_by_kind(&setting_node, "identifier")
      {
        let setting_name = self.get_node_text(&name_node);

        if !seen.insert(setting_name.clone()) {
          diagnostics.push(lsp::Diagnostic {
            range: self.node_to_range(&name_node),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!("Duplicate setting '{}'", setting_name),
            ..Default::default()
          });
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
      doc.find_child_by_kind_at_position(alias_node, "identifier", 1);

    assert!(alias_name.is_some());
    assert_eq!(doc.get_node_text(&alias_name.unwrap()), "foo");

    let target_name =
      doc.find_child_by_kind_at_position(alias_node, "identifier", 3);

    assert!(target_name.is_some());
    assert_eq!(doc.get_node_text(&target_name.unwrap()), "bar");
  }

  #[test]
  fn collect_parser_errors() {
    let doc = document(indoc! {
      "
      foo
        echo \"foo\"
      "
    });

    let diagnostics = doc.collect_parser_errors();

    assert!(!diagnostics.is_empty(), "Should detect syntax errors");

    let syntax_errors: Vec<_> = diagnostics
      .iter()
      .filter(|d| {
        d.message.contains("Syntax error")
          || d.message.contains("Missing syntax")
      })
      .collect();

    assert!(
      !syntax_errors.is_empty(),
      "Should have at least one syntax error diagnostic"
    );

    let valid_doc = document(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    let valid_diagnostics = valid_doc.collect_parser_errors();

    assert!(
      valid_diagnostics.is_empty(),
      "Valid document should not have syntax errors"
    );
  }

  #[test]
  fn validate_function_calls() {
    let doc = document(indoc! {
      "
      foo:
        echo {{ unknown_function() }}
      "
    });

    let diagnostics = doc.validate_function_calls();

    assert!(diagnostics.len() > 0, "Should have at least one diagnostic");

    assert!(
      diagnostics
        .iter()
        .any(|d| d.message.contains("Unknown function 'unknown_function'")),
      "Should have diagnostic about unknown function"
    );

    let doc = document(indoc! {
      "
      foo:
        echo {{ replace() }}
      "
    });

    let diagnostics = doc.validate_function_calls();

    assert!(diagnostics.len() > 0, "Should have at least one diagnostic");

    assert!(
      diagnostics
        .iter()
        .any(|d| d.message.contains("requires at least 3 argument(s)")),
      "Should have diagnostic about missing arguments"
    );

    let doc = document(indoc! {
      "
      foo:
        echo {{ uppercase(\"hello\", \"extra\") }}
      "
    });

    let diagnostics = doc.validate_function_calls();

    assert!(diagnostics.len() > 0, "Should have at least one diagnostic");

    assert!(
      diagnostics
        .iter()
        .any(|d| d.message.contains("accepts 1 argument(s)")),
      "Should have diagnostic about too many arguments"
    );

    let doc = document(indoc! {
      "
      foo:
        echo {{ arch() }}
        echo {{ join(\"a\", \"b\", \"c\") }}
      "
    });

    let diagnostics = doc.validate_function_calls();

    assert_eq!(
      diagnostics.len(),
      0,
      "Valid function calls should not produce diagnostics"
    );
  }

  #[test]
  fn validate_settings_unknown_setting() {
    let doc = document(indoc! {
      "
    set unknown-setting := true

    foo:
      echo \"foo\"
    "
    });

    let diagnostics = doc.validate_settings();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Unknown setting 'unknown-setting'");
  }

  #[test]
  fn validate_settings_boolean_type() {
    let doc = document(indoc! {
      "
    set export := 'foo'

    foo:
      echo \"foo\"
    "
    });

    let diagnostics = doc.validate_settings();

    assert_eq!(diagnostics.len(), 1);

    assert_eq!(
      diagnostics[0].message,
      "Setting 'export' expects a boolean value"
    );

    let doc = document(indoc! {
      "
      set export

      foo:
        echo \"foo\"
      "
    });

    let diagnostics = doc.validate_settings();

    assert_eq!(
      diagnostics.len(),
      0,
      "Shorthand boolean syntax should be valid"
    );

    let doc = document(indoc! {
      "
      set export := true
      set dotenv-load := false

      foo:
        echo \"foo\"
      "
    });

    let diagnostics = doc.validate_settings();

    assert_eq!(
      diagnostics.len(),
      0,
      "Valid boolean values should not produce diagnostics"
    );
  }

  #[test]
  fn validate_settings_string_type() {
    let doc = document(indoc! {
      "
      set dotenv-path := true

      foo:
        echo \"foo\"
      "
    });

    let diagnostics = doc.validate_settings();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
      diagnostics[0].message,
      "Setting 'dotenv-path' expects a string value"
    );

    let doc = document(indoc! {
      "
      set dotenv-path := \".env.development\"

      foo:
        echo \"foo\"
      "
    });

    let diagnostics = doc.validate_settings();

    assert_eq!(
      diagnostics.len(),
      0,
      "Valid string value should not produce diagnostics"
    );
  }

  #[test]
  fn validate_settings_duplicate() {
    let doc = document(indoc! {
      "
      set export := true
      set shell := [\"bash\", \"-c\"]
      set export := false

      foo:
        echo \"foo\"
      "
    });

    let diagnostics = doc.validate_settings();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Duplicate setting 'export'");
  }

  #[test]
  fn validate_settings_multiple_errors() {
    let doc = document(indoc! {
      "
      set unknown-setting := true
      set export := false
      set shell := ['bash']
      set export := false

      foo:
        echo \"foo\"
      "
    });

    let diagnostics = doc.validate_settings();

    assert_eq!(diagnostics.len(), 2, "Should detect all errors in settings");

    let messages: Vec<String> =
      diagnostics.iter().map(|d| d.message.clone()).collect();

    assert!(messages.contains(&"Unknown setting 'unknown-setting'".to_string()));

    assert!(messages.contains(&"Duplicate setting 'export'".to_string()));
  }
}
