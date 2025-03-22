use super::*;

#[derive(Debug)]
pub struct Resolver<'a> {
  document: &'a Document,
}

impl<'a> Resolver<'a> {
  pub(crate) fn new(document: &'a Document) -> Self {
    Self { document }
  }

  pub(crate) fn resolve_identifier(
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
              &candidate_parent.get_parent("recipe").map_or_else(
                String::new,
                |recipe_node| {
                  recipe_node.find("recipe_header > identifier").map_or_else(
                    String::new,
                    |identifier_node| {
                      self.document.get_node_text(&identifier_node)
                    },
                  )
                },
              ),
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
              &identifier.get_parent("recipe").map_or_else(
                String::new,
                |recipe_node| {
                  recipe_node.find("recipe_header > identifier").map_or_else(
                    String::new,
                    |identifier_node| {
                      self.document.get_node_text(&identifier_node)
                    },
                  )
                },
              ),
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
  fn resolve_recipe() {
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

    let references = resolver.resolve_identifier(&identifier);

    assert_eq!(references.len(), 3);
    assert_eq!(references[0].range.start.line, 0);
    assert_eq!(references[1].range.start.line, 3);
    assert_eq!(references[2].range.start.line, 6);
  }

  #[test]
  fn resolve_recipe_parameter() {
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

    let references = resolver.resolve_identifier(&identifier);

    assert_eq!(references.len(), 3);
    assert_eq!(references[0].range.start.line, 5);
    assert_eq!(references[1].range.start.line, 6);
    assert_eq!(references[2].range.start.line, 7);
  }

  #[test]
  fn resolve_interpolation() {
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

    let references = resolver.resolve_identifier(&identifier);

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

    let references = resolver.resolve_identifier(&identifier);

    assert_eq!(references.len(), 3);
    assert_eq!(references[0].range.start.line, 0);
    assert_eq!(references[1].range.start.line, 3);
  }

  #[test]
  fn resolve_variable() {
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

    let identifier = root.find("assignment > identifier").unwrap();

    let references = resolver.resolve_identifier(&identifier);

    assert_eq!(references.len(), 2);
    assert_eq!(references[0].range.start.line, 0);
    assert_eq!(references[1].range.start.line, 3);
  }

  #[test]
  fn resolve_dependency_argument() {
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

    let references = resolver.resolve_identifier(&identifier);

    assert_eq!(references.len(), 2);
    assert_eq!(references[0].range.start.line, 0);
    assert_eq!(references[1].range.start.line, 3);
  }
}
