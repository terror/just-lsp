use super::*;

#[derive(Debug)]
pub struct Resolver<'a> {
  document: &'a Document,
}

impl<'a> Resolver<'a> {
  pub(crate) fn new(document: &'a Document) -> Self {
    Self { document }
  }

  pub(crate) fn resolve_identifier_definition(
    &self,
    identifier: &Node,
  ) -> Option<lsp::Location> {
    let identifier_name = self.document.get_node_text(identifier);

    let identifier_parent_kind = identifier.parent()?.kind();

    if ["dependency", "alias"].contains(&identifier_parent_kind) {
      if let Some(recipe) = self.document.find_recipe(&identifier_name) {
        return Some(lsp::Location {
          uri: self.document.uri.clone(),
          range: recipe.range,
        });
      }
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

      let variables = self.document.get_variables();

      for variable in variables {
        if variable.name.value == identifier_name {
          return Some(lsp::Location {
            uri: self.document.uri.clone(),
            range: variable.range,
          });
        }
      }

      for builtin in builtins::BUILTINS {
        match builtin {
          Builtin::Constant { name, .. } if identifier_name == name => {
            return Some(lsp::Location {
              uri: self.document.uri.clone(),
              range: identifier.get_range(),
            })
          }
          _ => {}
        }
      }
    }

    for builtin in builtins::BUILTINS {
      match builtin {
        Builtin::Attribute { name, .. }
          if identifier_name == name
            && identifier_parent_kind == "attribute" =>
        {
          return Some(lsp::Location {
            uri: self.document.uri.clone(),
            range: identifier.get_range(),
          });
        }
        Builtin::Function { name, .. }
          if identifier_name == name
            && identifier_parent_kind == "function_call" =>
        {
          return Some(lsp::Location {
            uri: self.document.uri.clone(),
            range: identifier.get_range(),
          });
        }
        Builtin::Setting { name, .. }
          if identifier_name == name && identifier_parent_kind == "setting" =>
        {
          return Some(lsp::Location {
            uri: self.document.uri.clone(),
            range: identifier.get_range(),
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
            range: recipe_node.get_range(),
          });
        }
      }
      "assignment" => {
        return Some(lsp::Location {
          uri: self.document.uri.clone(),
          range: identifier.parent()?.get_range(),
        });
      }
      "parameter" | "variadic_parameter" => {
        return Some(lsp::Location {
          uri: self.document.uri.clone(),
          range: identifier.parent()?.get_range(),
        });
      }
      _ => {}
    }

    None
  }

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

        let candidate_parent = match candidate.parent() {
          Some(p) => p,
          None => return false,
        };

        let candidate_parent_kind = candidate_parent.kind();

        match identifier_parent_kind {
          "alias" | "recipe_header" => ["alias", "dependency", "recipe_header"]
            .contains(&candidate_parent_kind),
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
                .map(|identifier_node| {
                  self.document.get_node_text(&identifier_node)
                })
                .unwrap_or_else(String::new),
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
        range: found.get_range(),
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
    assert_eq!(references[0].range.start.line, 0);
    assert_eq!(references[1].range.start.line, 3);
    assert_eq!(references[2].range.start.line, 6);
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
    assert_eq!(references[0].range.start.line, 5);
    assert_eq!(references[1].range.start.line, 6);
    assert_eq!(references[2].range.start.line, 7);
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

    assert_eq!(references.len(), 2);
    assert_eq!(references[0].range.start.line, 2);
    assert_eq!(references[1].range.start.line, 3);

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
    assert_eq!(references[0].range.start.line, 0);
    assert_eq!(references[1].range.start.line, 3);
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
    assert_eq!(references[0].range.start.line, 0);
    assert_eq!(references[1].range.start.line, 3);
    assert_eq!(references[2].range.start.line, 10);
    assert_eq!(references[3].range.start.line, 11);
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
    assert_eq!(references[0].range.start.line, 0);
    assert_eq!(references[1].range.start.line, 3);
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
  fn resolve_builtin_identifier() {
    let doc = document(indoc! {
      "
      foo:
        echo {{ arch() }}
      "
    });

    let resolver = Resolver::new(&doc);

    let root = doc.tree.as_ref().unwrap().root_node();

    let builtin_usage = root.find("function_call > identifier").unwrap();

    let definition = resolver.resolve_identifier_definition(&builtin_usage);

    assert!(definition.is_some());
    assert_eq!(definition.unwrap().range, builtin_usage.get_range());
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
}
