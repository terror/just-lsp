use super::*;

#[derive(Debug)]
pub(crate) struct Resolver<'a> {
  document: &'a Document,
}

impl<'a> Resolver<'a> {
  #[must_use]
  pub(crate) fn new(document: &'a Document) -> Self {
    Self { document }
  }

  /// Returns the [`lsp::Location`] that defines the symbol represented by
  /// `identifier`.
  ///
  /// The resolver inspects the identifier's parent node to infer what kind of
  /// symbol is being referenced. Recipe invocations that appear in aliases or
  /// dependency lists resolve to the recipe header, identifiers inside a
  /// recipe body (`value`) prefer matching parameters, then global variables,
  /// and finally builtin constants. It also surfaces builtin
  /// attribute/function/setting identifiers by returning the range of
  /// the identifier itself so editors can jump to inline documentation.
  ///
  /// When the identifier belongs to a header construct (assignments,
  /// parameters, variadic parameters, or the recipe header identifier) the
  /// range of that construct is returned. `None` indicates that the resolver
  /// could not find a definition in the current document.
  #[must_use]
  pub(crate) fn resolve_identifier_definition(
    &self,
    identifier: &Node,
  ) -> Option<lsp::Location> {
    let identifier_name = self.document.get_node_text(identifier);

    let identifier_parent_kind = identifier.parent()?.kind();

    if ["dependency", "alias"].contains(&identifier_parent_kind)
      && let Some(recipe) = self.document.find_recipe(&identifier_name)
    {
      return Some(lsp::Location {
        uri: self.document.uri.clone(),
        range: recipe.range,
      });
    }

    if identifier_parent_kind == "value" {
      let recipe_node = identifier.get_parent("recipe")?;

      let recipe = self.document.find_recipe(
        &self
          .document
          .get_node_text(&recipe_node.find("recipe_header > identifier")?),
      );

      if let Some(recipe) = recipe {
        for param in &recipe.parameters {
          if param.name == identifier_name {
            return Some(lsp::Location {
              uri: self.document.uri.clone(),
              range: param.range,
            });
          }
        }
      }

      let variables = self.document.variables();

      for variable in variables {
        if variable.name.value == identifier_name {
          return Some(lsp::Location {
            uri: self.document.uri.clone(),
            range: variable.range,
          });
        }
      }

      for builtin in BUILTINS {
        match builtin {
          Builtin::Constant { name, .. } if identifier_name == name => {
            return Some(lsp::Location {
              uri: self.document.uri.clone(),
              range: identifier.get_range(self.document),
            });
          }
          _ => {}
        }
      }
    }

    for builtin in BUILTINS {
      match builtin {
        Builtin::Attribute { name, .. }
          if identifier_name == name
            && identifier_parent_kind == "attribute" =>
        {
          return Some(lsp::Location {
            uri: self.document.uri.clone(),
            range: identifier.get_range(self.document),
          });
        }
        Builtin::Function { name, .. }
          if identifier_name == name
            && identifier_parent_kind == "function_call" =>
        {
          return Some(lsp::Location {
            uri: self.document.uri.clone(),
            range: identifier.get_range(self.document),
          });
        }
        Builtin::Setting { name, .. }
          if identifier_name == name && identifier_parent_kind == "setting" =>
        {
          return Some(lsp::Location {
            uri: self.document.uri.clone(),
            range: identifier.get_range(self.document),
          });
        }
        _ => {}
      }
    }

    match identifier_parent_kind {
      "recipe_header" => {
        let recipe_node = identifier.parent()?.parent()?;

        if recipe_node.kind() == "recipe" {
          return Some(lsp::Location {
            uri: self.document.uri.clone(),
            range: recipe_node.get_range(self.document),
          });
        }
      }
      "assignment" | "parameter" | "variadic_parameter" => {
        return Some(lsp::Location {
          uri: self.document.uri.clone(),
          range: identifier.parent()?.get_range(self.document),
        });
      }
      _ => {}
    }

    None
  }

  /// Builds an [`lsp::Hover`] for the symbol at `identifier`.
  ///
  /// When the identifier names a recipe (inside aliases, dependencies, or the
  /// recipe header) the hover shows the rendered recipe body. Within a recipe
  /// body the resolver prefers parameter documentation, then global variable
  /// documentation, and finally the associated builtin constant description.
  /// Builtin attributes, functions, and settings also map to their builtin
  /// documentation as long as the identifier appears in the appropriate
  /// syntactic context.
  ///
  /// If no contextual information exists the function returns `None`.
  #[must_use]
  pub(crate) fn resolve_identifier_hover(
    &self,
    identifier: &Node,
  ) -> Option<lsp::Hover> {
    let text = self.document.get_node_text(identifier);

    let parent_kind = identifier.parent().map(|p| p.kind());

    if let Some(recipe) = self.document.find_recipe(&text)
      && parent_kind.is_some_and(|kind| {
        ["alias", "dependency", "recipe_header"].contains(&kind)
      })
    {
      return Some(lsp::Hover {
        contents: lsp::HoverContents::Markup(lsp::MarkupContent {
          kind: lsp::MarkupKind::PlainText,
          value: recipe.content,
        }),
        range: Some(identifier.get_range(self.document)),
      });
    }

    if parent_kind.is_some_and(|kind| kind == "value") {
      let recipe_node = identifier.get_parent("recipe")?;

      let recipe = self.document.find_recipe(
        &self
          .document
          .get_node_text(&recipe_node.find("recipe_header > identifier")?),
      );

      if let Some(recipe) = recipe {
        for parameter in recipe.parameters {
          if parameter.name == text {
            return Some(lsp::Hover {
              contents: lsp::HoverContents::Markup(lsp::MarkupContent {
                kind: lsp::MarkupKind::PlainText,
                value: parameter.content,
              }),
              range: Some(identifier.get_range(self.document)),
            });
          }
        }
      }

      let variables = self.document.variables();

      for variable in variables {
        if variable.name.value == text {
          return Some(lsp::Hover {
            contents: lsp::HoverContents::Markup(lsp::MarkupContent {
              kind: lsp::MarkupKind::PlainText,
              value: variable.content,
            }),
            range: Some(identifier.get_range(self.document)),
          });
        }
      }

      for builtin in BUILTINS {
        match builtin {
          Builtin::Constant { name, .. } if text == name => {
            return Some(lsp::Hover {
              contents: lsp::HoverContents::Markup(builtin.documentation()),
              range: Some(identifier.get_range(self.document)),
            });
          }
          _ => {}
        }
      }
    }

    for builtin in BUILTINS {
      match builtin {
        Builtin::Attribute { name, .. }
          if text == name
            && parent_kind.is_some_and(|kind| kind == "attribute") =>
        {
          return Some(lsp::Hover {
            contents: lsp::HoverContents::Markup(builtin.documentation()),
            range: Some(identifier.get_range(self.document)),
          });
        }
        Builtin::Function { name, .. }
          if text == name
            && parent_kind.is_some_and(|kind| kind == "function_call") =>
        {
          return Some(lsp::Hover {
            contents: lsp::HoverContents::Markup(builtin.documentation()),
            range: Some(identifier.get_range(self.document)),
          });
        }
        Builtin::Setting { name, .. }
          if text == name
            && parent_kind.is_some_and(|kind| kind == "setting") =>
        {
          return Some(lsp::Hover {
            contents: lsp::HoverContents::Markup(builtin.documentation()),
            range: Some(identifier.get_range(self.document)),
          });
        }
        _ => {}
      }
    }

    None
  }

  /// Collects every [`lsp::Location`] that references the same logical symbol
  /// as `identifier`.
  ///
  /// The resolver walks the entire syntax tree, filters identifier nodes with
  /// matching text, and then applies parent-kind specific rules so that
  /// references stay within the correct scope. Recipe names only match other
  /// aliases/dependencies/headers, assignment targets only match body usages
  /// that are not shadowed by parameters, and parameters (including variadic
  /// ones) only match within the same recipe. Identifiers inside the
  /// recipe body (`value`) will match local parameters first and fall back to
  /// global assignments when no shadowing parameters exist. The identifier
  /// node itself is always included.
  ///
  /// An empty vector is returned when the document tree or necessary parent
  /// context is missing.
  #[must_use]
  pub(crate) fn resolve_identifier_references(
    &self,
    identifier: &Node,
  ) -> Vec<lsp::Location> {
    let identifier_name = self.document.get_node_text(identifier);

    let identifier_parent_kind = match identifier.parent() {
      Some(parent) => parent.kind(),
      None => return Vec::new(),
    };

    let root = match &self.document.tree {
      Some(tree) => tree.root_node(),
      None => return Vec::new(),
    };

    root
      .find_all("identifier")
      .into_iter()
      .filter(|candidate| {
        if candidate.id() == identifier.id() {
          return true;
        }

        if self.document.get_node_text(candidate) != identifier_name {
          return false;
        }

        let Some(candidate_parent) = candidate.parent() else {
          return false;
        };

        let candidate_parent_kind = candidate_parent.kind();

        match identifier_parent_kind {
          "alias" | "dependency" | "recipe_header" => {
            ["alias", "dependency", "recipe_header"]
              .contains(&candidate_parent_kind)
          }
          "assignment" => {
            if candidate_parent_kind != "value" {
              return false;
            }

            let candidate_recipe = self.document.find_recipe(
              &candidate_parent
                .get_parent("recipe")
                .as_ref()
                .and_then(|recipe_node| {
                  recipe_node.find("recipe_header > identifier")
                })
                .map_or_else(String::new, |identifier_node| {
                  self.document.get_node_text(&identifier_node)
                }),
            );

            candidate_recipe.is_some_and(|recipe| {
              !recipe
                .parameters
                .iter()
                .any(|param| param.name == identifier_name)
            })
          }
          "parameter" | "variadic_parameter" => {
            let in_same_recipe = match (
              identifier.get_parent("recipe"),
              candidate.get_parent("recipe"),
            ) {
              (Some(r1), Some(r2)) => r1.id() == r2.id(),
              _ => false,
            };

            in_same_recipe
              && ["value", "parameter", "variadic_parameter"]
                .contains(&candidate_parent_kind)
          }
          "value" => {
            let in_same_recipe = match (
              identifier.get_parent("recipe"),
              candidate.get_parent("recipe"),
            ) {
              (Some(r1), Some(r2)) => r1.id() == r2.id(),
              _ => false,
            };

            if in_same_recipe
              && ["parameter", "value"].contains(&candidate_parent_kind)
            {
              return true;
            }

            let identifier_recipe = self.document.find_recipe(
              &identifier
                .get_parent("recipe")
                .as_ref()
                .and_then(|recipe_node| {
                  recipe_node.find("recipe_header > identifier")
                })
                .map(|identifier_node| {
                  self.document.get_node_text(&identifier_node)
                })
                .unwrap_or_default(),
            );

            identifier_recipe.is_some_and(|recipe| {
              candidate_parent_kind == "assignment"
                && !recipe
                  .parameters
                  .iter()
                  .any(|param| param.name == identifier_name)
            })
          }
          _ => false,
        }
      })
      .map(|found| lsp::Location {
        uri: self.document.uri.clone(),
        range: found.get_range(self.document),
      })
      .collect()
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
  fn resolve_recipe_references() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar foo: foo
        echo \"bar\"

      alias baz := foo
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let identifier = root.find("recipe_header > identifier").unwrap();

    let references = resolver.resolve_identifier_references(&identifier);

    assert_eq!(references.len(), 3);

    let ranges = references
      .iter()
      .map(|reference| reference.range)
      .collect::<Vec<_>>();

    assert_eq!(
      ranges,
      vec![
        lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 0,
            character: 3
          },
        },
        lsp::Range {
          start: lsp::Position {
            line: 3,
            character: 9
          },
          end: lsp::Position {
            line: 3,
            character: 12
          },
        },
        lsp::Range {
          start: lsp::Position {
            line: 6,
            character: 13
          },
          end: lsp::Position {
            line: 6,
            character: 16
          },
        },
      ]
    );
  }

  #[test]
  fn resolve_recipe_parameter_references() {
    let doc = document(indoc! {
      "
      foo := 'bar'

      foo:
        echo {{ foo }}

      bar foo: foo
        echo {{ foo }}
        echo {{ foo }}

      alias baz := foo
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let identifier = root.find("parameter > identifier").unwrap();

    let references = resolver.resolve_identifier_references(&identifier);

    assert_eq!(references.len(), 3);

    let ranges = references
      .iter()
      .map(|reference| reference.range)
      .collect::<Vec<_>>();

    assert_eq!(
      ranges,
      vec![
        lsp::Range {
          start: lsp::Position {
            line: 5,
            character: 4
          },
          end: lsp::Position {
            line: 5,
            character: 7
          },
        },
        lsp::Range {
          start: lsp::Position {
            line: 6,
            character: 10
          },
          end: lsp::Position {
            line: 6,
            character: 13
          },
        },
        lsp::Range {
          start: lsp::Position {
            line: 7,
            character: 10
          },
          end: lsp::Position {
            line: 7,
            character: 13
          },
        },
      ]
    );
  }

  #[test]
  fn resolve_value_references() {
    let doc = document(indoc! {
      "
      foo := \"foo\"

      foo foo:
        echo {{ foo }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let identifier = root.find("value > identifier").unwrap();

    let references = resolver.resolve_identifier_references(&identifier);

    let ranges = references
      .iter()
      .map(|reference| reference.range)
      .collect::<Vec<_>>();

    assert_eq!(references.len(), 2);

    assert_eq!(
      ranges,
      vec![
        lsp::Range {
          start: lsp::Position {
            line: 2,
            character: 4
          },
          end: lsp::Position {
            line: 2,
            character: 7
          },
        },
        lsp::Range {
          start: lsp::Position {
            line: 3,
            character: 10
          },
          end: lsp::Position {
            line: 3,
            character: 13
          },
        },
      ]
    );

    let doc = document(indoc! {
      "
      foo := \"foo\"

      foo:
        echo {{ foo / foo }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let identifier = root.find("value > identifier").unwrap();

    let references = resolver.resolve_identifier_references(&identifier);

    assert_eq!(references.len(), 3);

    let ranges = references
      .iter()
      .map(|reference| reference.range)
      .collect::<Vec<_>>();

    assert_eq!(
      ranges,
      vec![
        lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 0,
            character: 3
          },
        },
        lsp::Range {
          start: lsp::Position {
            line: 3,
            character: 10
          },
          end: lsp::Position {
            line: 3,
            character: 13
          },
        },
        lsp::Range {
          start: lsp::Position {
            line: 3,
            character: 16
          },
          end: lsp::Position {
            line: 3,
            character: 19
          },
        },
      ]
    );
  }

  #[test]
  fn resolve_variable_references() {
    let doc = document(indoc! {
      "
      foo := 'bar'

      foo:
        echo {{ foo }}

      bar foo: foo
        echo {{ foo }}
        echo {{ foo }}

      quux:
        echo {{ foo }}
        echo {{ foo }}

      alias baz := foo
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let identifier = root.find("assignment > identifier").unwrap();

    let references = resolver.resolve_identifier_references(&identifier);

    assert_eq!(references.len(), 4);

    let ranges = references
      .iter()
      .map(|reference| reference.range)
      .collect::<Vec<_>>();

    assert_eq!(
      ranges,
      vec![
        lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 0,
            character: 3
          },
        },
        lsp::Range {
          start: lsp::Position {
            line: 3,
            character: 10
          },
          end: lsp::Position {
            line: 3,
            character: 13
          },
        },
        lsp::Range {
          start: lsp::Position {
            line: 10,
            character: 10
          },
          end: lsp::Position {
            line: 10,
            character: 13
          },
        },
        lsp::Range {
          start: lsp::Position {
            line: 11,
            character: 10
          },
          end: lsp::Position {
            line: 11,
            character: 13
          },
        },
      ]
    );
  }

  #[test]
  fn resolve_dependency_references() {
    let doc = document(indoc! {
      "
      all: foo

      foo:
        echo foo
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let identifier = root.find("dependency > identifier").unwrap();

    let references = resolver.resolve_identifier_references(&identifier);

    assert_eq!(references.len(), 2);

    let ranges = references
      .iter()
      .map(|reference| reference.range)
      .collect::<Vec<_>>();

    assert_eq!(
      ranges,
      vec![
        lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 5
          },
          end: lsp::Position {
            line: 0,
            character: 8
          },
        },
        lsp::Range {
          start: lsp::Position {
            line: 2,
            character: 0
          },
          end: lsp::Position {
            line: 2,
            character: 3
          },
        },
      ]
    );
  }

  #[test]
  fn resolve_dependency_argument_references() {
    let doc = document(indoc! {
      "
      a := 'foo'

      [group: 'test']
      foo: (bar a)

      bar a:
        echo {{ a }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let identifier = root
      .find("dependency_expression > expression > value > identifier")
      .unwrap();

    let references = resolver.resolve_identifier_references(&identifier);

    assert_eq!(references.len(), 2);

    let ranges = references
      .iter()
      .map(|reference| reference.range)
      .collect::<Vec<_>>();

    assert_eq!(
      ranges,
      vec![
        lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0
          },
          end: lsp::Position {
            line: 0,
            character: 1
          },
        },
        lsp::Range {
          start: lsp::Position {
            line: 3,
            character: 10
          },
          end: lsp::Position {
            line: 3,
            character: 11
          },
        },
      ]
    );
  }

  #[test]
  fn resolve_recipe_definition() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let foo_dependency = root.find("dependency > identifier").unwrap();

    let definition = resolver
      .resolve_identifier_definition(&foo_dependency)
      .unwrap();

    assert_eq!(
      definition.range,
      lsp::Range {
        start: lsp::Position {
          line: 0,
          character: 0
        },
        end: lsp::Position {
          line: 3,
          character: 0
        },
      }
    );
  }

  #[test]
  fn resolve_variable_definition() {
    let doc = document(indoc! {
      "
      var := \"value\"

      foo:
        echo {{ var }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let var_usage = root.find("value > identifier").unwrap();

    let definition =
      resolver.resolve_identifier_definition(&var_usage).unwrap();

    assert_eq!(
      definition.range,
      lsp::Range {
        start: lsp::Position {
          line: 0,
          character: 0
        },
        end: lsp::Position {
          line: 1,
          character: 0
        },
      }
    );
  }

  #[test]
  fn resolve_parameter_definition() {
    let doc = document(indoc! {
      "
      foo param=\"default\":
        echo {{ param }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let param_usage = root.find("value > identifier").unwrap();

    let definition = resolver
      .resolve_identifier_definition(&param_usage)
      .unwrap();

    assert_eq!(
      definition.range,
      lsp::Range {
        start: lsp::Position {
          line: 0,
          character: 4
        },
        end: lsp::Position {
          line: 0,
          character: 19
        },
      }
    );
  }

  #[test]
  fn resolve_builtin_identifier_definition() {
    let doc = document(indoc! {
      "
      foo:
        echo {{ arch() }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let builtin_usage = root.find("function_call > identifier").unwrap();

    let definition = resolver
      .resolve_identifier_definition(&builtin_usage)
      .unwrap();

    assert_eq!(definition.range, builtin_usage.get_range(&doc));
  }

  #[test]
  fn resolve_self_definition() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let recipe_name = root.find("recipe_header > identifier").unwrap();

    let definition = resolver
      .resolve_identifier_definition(&recipe_name)
      .unwrap();

    assert_eq!(
      definition.range,
      lsp::Range {
        start: lsp::Position {
          line: 0,
          character: 0
        },
        end: lsp::Position {
          line: 2,
          character: 0
        },
      }
    );
  }

  #[test]
  fn resolve_recipe_hover() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let hover = resolver
      .resolve_identifier_hover(
        &root.find("recipe_header > identifier").unwrap(),
      )
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::PlainText,
        value: "foo:\n  echo \"foo\"".to_string(),
      })
    );

    let dependency = root.find("dependency > identifier").unwrap();

    let hover = resolver.resolve_identifier_hover(&dependency).unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::PlainText,
        value: "foo:\n  echo \"foo\"".to_string(),
      })
    );
  }

  #[test]
  fn resolve_recipe_hover_in_alias() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      alias f := foo
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let hover = resolver
      .resolve_identifier_hover(&root.find("alias > identifier[1]").unwrap())
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::PlainText,
        value: "foo:\n  echo \"foo\"".to_string(),
      })
    );
  }

  #[test]
  fn resolve_parameter_hover() {
    let doc = document(indoc! {
      "
      foo param=\"default\":
        echo {{ param }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let hover = resolver
      .resolve_identifier_hover(&root.find("value > identifier").unwrap())
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::PlainText,
        value: "param=\"default\"".to_string(),
      })
    );
  }

  #[test]
  fn resolve_variadic_parameter_hover() {
    let doc = document(indoc! {
      "
      foo +args:
        echo {{ args }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let hover = resolver
      .resolve_identifier_hover(&root.find("value > identifier").unwrap())
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::PlainText,
        value: "+args".to_string(),
      })
    );
  }

  #[test]
  fn resolve_export_parameter_hover() {
    let doc = document(indoc! {
      "
      foo $env_var:
        echo {{ env_var }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let hover = resolver
      .resolve_identifier_hover(&root.find("value > identifier").unwrap())
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::PlainText,
        value: "$env_var".to_string(),
      })
    );
  }

  #[test]
  fn resolve_variable_hover() {
    let doc = document(indoc! {
      "
      var := \"value\"

      foo:
        echo {{ var }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let hover = resolver
      .resolve_identifier_hover(&root.find("value > identifier").unwrap())
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::PlainText,
        value: "var := \"value\"".to_string(),
      })
    );
  }

  #[test]
  fn resolve_export_variable_hover() {
    let doc = document(indoc! {
      "
      export VERSION := \"1.0.0\"

      foo:
        echo {{ VERSION }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let hover = resolver
      .resolve_identifier_hover(&root.find("value > identifier").unwrap())
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::PlainText,
        value: "VERSION := \"1.0.0\"".to_string(),
      })
    );
  }

  #[test]
  fn resolve_builtin_function_hover() {
    let doc = document(indoc! {
      "
      foo:
        echo {{ arch() }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let hover = resolver
      .resolve_identifier_hover(
        &root.find("function_call > identifier").unwrap(),
      )
      .unwrap();

    assert!(matches!(hover.contents, lsp::HoverContents::Markup(_)));

    if let lsp::HoverContents::Markup(content) = hover.contents {
      assert_eq!(content.kind, lsp::MarkupKind::Markdown);
      assert!(content.value.contains("arch"));
      assert!(content.value.contains("Instruction set architecture"));
    }
  }

  #[test]
  fn resolve_builtin_constant_hover() {
    let doc = document(indoc! {
      "
      foo:
        echo {{ RED }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let hover = resolver
      .resolve_identifier_hover(&root.find("value > identifier").unwrap())
      .unwrap();

    assert!(matches!(hover.contents, lsp::HoverContents::Markup(_)));

    if let lsp::HoverContents::Markup(content) = hover.contents {
      assert_eq!(content.kind, lsp::MarkupKind::Markdown);
      assert!(content.value.contains("Red text"));
    }
  }

  #[test]
  fn resolve_builtin_attribute_hover() {
    let doc = document(indoc! {
      "
      [no-cd]
      foo:
        echo \"foo\"
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let hover = resolver
      .resolve_identifier_hover(&root.find("attribute > identifier").unwrap())
      .unwrap();

    assert!(matches!(hover.contents, lsp::HoverContents::Markup(_)));

    if let lsp::HoverContents::Markup(content) = hover.contents {
      assert_eq!(content.kind, lsp::MarkupKind::Markdown);
      assert!(content.value.contains("no-cd"));
      assert!(content.value.contains("Don't change directory"));
    }
  }

  #[test]
  fn resolve_builtin_setting_hover() {
    let doc = document(indoc! {
      "
      set export

      foo:
        echo \"foo\"
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let hover = resolver
      .resolve_identifier_hover(&root.find("setting > identifier").unwrap())
      .unwrap();

    assert!(matches!(hover.contents, lsp::HoverContents::Markup(_)));

    if let lsp::HoverContents::Markup(content) = hover.contents {
      assert_eq!(content.kind, lsp::MarkupKind::Markdown);
      assert!(content.value.contains("export"));
    }
  }

  #[test]
  fn resolve_same_name_confusion() {
    let doc = document(indoc! {
      "
      arch := \"custom_arch\"

      foo:
        echo {{ arch }}
        echo {{ arch() }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let hover = resolver
      .resolve_identifier_hover(&root.find("value > identifier").unwrap())
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::PlainText,
        value: "arch := \"custom_arch\"".to_string(),
      })
    );

    let hover = resolver
      .resolve_identifier_hover(
        &root.find("function_call > identifier").unwrap(),
      )
      .unwrap();

    assert!(matches!(hover.contents, lsp::HoverContents::Markup(_)));

    if let lsp::HoverContents::Markup(content) = hover.contents {
      assert_eq!(content.kind, lsp::MarkupKind::Markdown);
      assert!(content.value.contains("Instruction set architecture"));
    }
  }

  #[test]
  fn resolve_parameter_over_variable() {
    let doc = document(indoc! {
      "
      param := \"global value\"

      foo param=\"local value\":
        echo {{ param }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let hover = resolver
      .resolve_identifier_hover(&root.find("value > identifier").unwrap())
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::PlainText,
        value: "param=\"local value\"".to_string(),
      })
    );
  }

  #[test]
  fn resolve_hover_non_identifier() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    assert!(
      resolver
        .resolve_identifier_hover(&root.find("text").unwrap())
        .is_none()
    );
  }

  #[test]
  fn resolve_hover_nonexistent_variable() {
    let doc = document(indoc! {
      "
      foo:
        echo {{ nonexistent }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let nonexistent = root.find("value > identifier").unwrap();

    assert!(resolver.resolve_identifier_hover(&nonexistent).is_none());
  }
}
