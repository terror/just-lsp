use super::*;

#[derive(Debug)]
pub(crate) struct Resolver<'a> {
  document: &'a Document,
}

impl<'a> Resolver<'a> {
  /// Creates a new `Resolver` bound to the given `Document`.
  #[must_use]
  pub(crate) fn new(document: &'a Document) -> Self {
    Self { document }
  }

  /// Returns the definition site of the symbol that `identifier` refers
  /// to. Builtins have no in-document declaration, so the identifier's
  /// own range is returned instead, letting editors anchor inline
  /// documentation at the cursor.
  #[must_use]
  pub(crate) fn resolve_identifier_definition(
    &self,
    identifier: &Node,
  ) -> Option<lsp::Location> {
    Some(lsp::Location {
      range: match self.resolve_symbol(identifier)? {
        Symbol::Builtin(_) => identifier.get_range(self.document),
        Symbol::Parameter(parameter) => parameter.range,
        Symbol::Recipe(recipe) => recipe.range,
        Symbol::Variable(variable) => variable.range,
      },
      uri: self.document.uri.clone(),
    })
  }

  /// Builds hover content for the symbol at `identifier`. User-defined
  /// symbols show their source text; builtins show their Markdown
  /// documentation from the static [`BUILTINS`] table.
  #[must_use]
  pub(crate) fn resolve_identifier_hover(
    &self,
    identifier: &Node,
  ) -> Option<lsp::Hover> {
    Some(lsp::Hover {
      contents: lsp::HoverContents::Markup(
        match self.resolve_symbol(identifier)? {
          Symbol::Builtin(builtin) => builtin.documentation(),
          Symbol::Parameter(parameter) => lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: parameter.content,
          },
          Symbol::Recipe(recipe) => lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: recipe.content,
          },
          Symbol::Variable(variable) => lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: variable.content,
          },
        },
      ),
      range: Some(identifier.get_range(self.document)),
    })
  }

  /// Collects every location in the document that references the same
  /// symbol as `identifier`. The identifier itself is always included.
  ///
  /// Scoping follows `just`'s semantics: parameters are local to their
  /// recipe, while variables are global but can be shadowed by a
  /// same-named parameter. Variable references inside parameter defaults
  /// (e.g. `a=a`) are treated as belonging to the outer scope, not the
  /// parameter being defined.
  #[must_use]
  pub(crate) fn resolve_identifier_references(
    &self,
    identifier: &Node,
  ) -> Vec<lsp::Location> {
    let name = self.document.get_node_text(identifier);

    let Some(symbol) = self.resolve_symbol(identifier) else {
      return Vec::new();
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

        if self.document.get_node_text(candidate) != name {
          return false;
        }

        let Some(candidate_parent) = candidate.parent() else {
          return false;
        };

        let candidate_parent_kind = candidate_parent.kind();

        match &symbol {
          Symbol::Builtin(_) => false,
          Symbol::Parameter(_) => {
            let in_same_recipe = matches!(
              (identifier.get_parent("recipe"), candidate.get_parent("recipe")),
              (Some(r1), Some(r2)) if r1.id() == r2.id()
            );

            in_same_recipe
              && ["value", "parameter", "variadic_parameter"]
                .contains(&candidate_parent_kind)
          }
          Symbol::Recipe(_) => [
            "alias",
            "dependency",
            "dependency_expression",
            "recipe_header",
          ]
          .contains(&candidate_parent_kind),

          Symbol::Variable(_) => {
            if candidate_parent_kind == "assignment" {
              return true;
            }

            if candidate_parent_kind != "value" {
              return false;
            }

            if candidate.get_parent("parameter").is_some() {
              return true;
            }

            candidate.get_recipe(self.document).is_some_and(|recipe| {
              !recipe
                .parameters
                .iter()
                .any(|parameter| parameter.name == name)
            })
          }
        }
      })
      .map(|found| lsp::Location {
        uri: self.document.uri.clone(),
        range: found.get_range(self.document),
      })
      .collect()
  }

  /// Classifies `identifier` into the [`Symbol`] it refers to, following
  /// `just`'s name-resolution priority: recipe names, then parameters
  /// (which shadow globals within their recipe), then variables, then
  /// builtins.
  ///
  /// Identifiers at definition sites (the left-hand side of an
  /// assignment, or a parameter name in a recipe header) are looked up
  /// through the document so that callers receive a fully-populated
  /// [`Symbol`] rather than a raw range.
  fn resolve_symbol(&self, identifier: &Node) -> Option<Symbol> {
    let name = self.document.get_node_text(identifier);

    let parent_kind = identifier.parent()?.kind();

    let builtin_constant = |name: &str| {
      BUILTINS
        .iter()
        .find(|builtin| matches!(
          builtin,
          Builtin::Constant { name: builtin_name, .. } if name == *builtin_name
        ))
        .map(Symbol::Builtin)
    };

    match parent_kind {
      "alias" | "dependency" | "dependency_expression" | "recipe_header" => {
        self.document.find_recipe(&name).map(Symbol::Recipe)
      }
      "parameter" | "variadic_parameter" => {
        identifier.get_recipe(self.document).and_then(|recipe| {
          recipe
            .parameters
            .iter()
            .find(|parameter| parameter.name == name)
            .cloned()
            .map(Symbol::Parameter)
        })
      }
      "assignment" => self.document.find_variable(&name).map(Symbol::Variable),
      "value" if identifier.get_parent("parameter").is_none() => identifier
        .get_recipe(self.document)
        .and_then(|recipe| {
          recipe
            .parameters
            .iter()
            .find(|parameter| parameter.name == name)
            .cloned()
            .map(Symbol::Parameter)
        })
        .or_else(|| self.document.find_variable(&name).map(Symbol::Variable))
        .or_else(|| builtin_constant(&name)),
      "value" => self
        .document
        .find_variable(&name)
        .map(Symbol::Variable)
        .or_else(|| builtin_constant(&name)),
      _ => BUILTINS
        .iter()
        .find(|builtin| match builtin {
          Builtin::Attribute {
            name: attribute_name,
            ..
          } => parent_kind == "attribute" && name == *attribute_name,
          Builtin::Constant { .. } => false,
          Builtin::Function {
            name: function_name,
            ..
          } => parent_kind == "function_call" && name == *function_name,
          Builtin::Setting {
            name: setting_name, ..
          } => parent_kind == "setting" && name == *setting_name,
        })
        .map(Symbol::Builtin),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  #[test]
  fn resolve_recipe_references() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      bar foo: foo
        echo \"bar\"

      alias baz := foo
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
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

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      foo := \"foo\"

      foo foo:
        echo {{ foo }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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

    let document = Document::from(indoc! {
      "
      foo := \"foo\"

      foo:
        echo {{ foo / foo }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
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

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
  fn resolve_shadowed_parameter_default_references() {
    let document = Document::from(indoc! {
      "
      a := 'foo'

      b a=a:
        echo {{ a }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

    let identifier = root.find("assignment > identifier").unwrap();

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
            character: 0,
          },
          end: lsp::Position {
            line: 0,
            character: 1,
          },
        },
        lsp::Range {
          start: lsp::Position {
            line: 2,
            character: 4,
          },
          end: lsp::Position {
            line: 2,
            character: 5,
          },
        },
      ]
    );
  }

  #[test]
  fn resolve_shadowed_parameter_default_definition() {
    let document = Document::from(indoc! {
      "
      a := 'foo'

      b a=a:
        echo {{ a }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

    let identifier = root.find("parameter > value > identifier").unwrap();

    let definition =
      resolver.resolve_identifier_definition(&identifier).unwrap();

    assert_eq!(
      definition.range,
      lsp::Range {
        start: lsp::Position {
          line: 0,
          character: 0,
        },
        end: lsp::Position {
          line: 1,
          character: 0,
        },
      }
    );
  }

  #[test]
  fn resolve_dependency_references() {
    let document = Document::from(indoc! {
      "
      all: foo

      foo:
        echo foo
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      a := 'foo'

      [group: 'test']
      foo: (bar a)

      bar a:
        echo {{ a }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
  fn resolve_dependency_expression_definition() {
    let document = Document::from(indoc! {
      "
      foo:
        echo foo

      bar: (foo)
        echo bar
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

    let identifier = root.find("dependency_expression > identifier").unwrap();

    let definition =
      resolver.resolve_identifier_definition(&identifier).unwrap();

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
  fn resolve_dependency_expression_references() {
    let document = Document::from(indoc! {
      "
      foo:
        echo foo

      bar: (foo)
        echo bar
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

    let identifier = root.find("dependency_expression > identifier").unwrap();

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
            character: 3
          },
        },
        lsp::Range {
          start: lsp::Position {
            line: 3,
            character: 6
          },
          end: lsp::Position {
            line: 3,
            character: 9
          },
        },
      ]
    );
  }

  #[test]
  fn resolve_dependency_expression_hover() {
    let document = Document::from(indoc! {
      "
      foo:
        echo foo

      bar: (foo)
        echo bar
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

    let identifier = root.find("dependency_expression > identifier").unwrap();

    let hover = resolver.resolve_identifier_hover(&identifier).unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::PlainText,
        value: "foo:\n  echo foo".to_string(),
      })
    );
  }

  #[test]
  fn resolve_recipe_definition() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      var := \"value\"

      foo:
        echo {{ var }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      foo param=\"default\":
        echo {{ param }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      foo:
        echo {{ arch() }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

    let builtin_usage = root.find("function_call > identifier").unwrap();

    let definition = resolver
      .resolve_identifier_definition(&builtin_usage)
      .unwrap();

    assert_eq!(definition.range, builtin_usage.get_range(&document));
  }

  #[test]
  fn resolve_self_definition() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      alias f := foo
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      foo param=\"default\":
        echo {{ param }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      foo +args:
        echo {{ args }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      foo $env_var:
        echo {{ env_var }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      var := \"value\"

      foo:
        echo {{ var }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      export VERSION := \"1.0.0\"

      foo:
        echo {{ VERSION }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      foo:
        echo {{ arch() }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      foo:
        echo {{ RED }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      [no-cd]
      foo:
        echo \"foo\"
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      set export

      foo:
        echo \"foo\"
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      arch := \"custom_arch\"

      foo:
        echo {{ arch }}
        echo {{ arch() }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      param := \"global value\"

      foo param=\"local value\":
        echo {{ param }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

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
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

    assert!(
      resolver
        .resolve_identifier_hover(&root.find("text").unwrap())
        .is_none()
    );
  }

  #[test]
  fn resolve_hover_nonexistent_variable() {
    let document = Document::from(indoc! {
      "
      foo:
        echo {{ nonexistent }}
      "
    });

    let resolver = Resolver::new(&document);

    let root = document.tree.as_ref().unwrap().root_node();

    let nonexistent = root.find("value > identifier").unwrap();

    assert!(resolver.resolve_identifier_hover(&nonexistent).is_none());
  }
}
