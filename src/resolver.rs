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
        Symbol::Function(function) => function.name.range,
        Symbol::FunctionParameter(parameter) => parameter.range,
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
          Symbol::Function(function) => lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: function.content,
          },
          Symbol::FunctionParameter(parameter) => lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: parameter.value,
          },
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
          Symbol::Function(_) => {
            candidate_parent_kind == "function_call"
              || candidate_parent_kind == "function_definition"
          }
          Symbol::FunctionParameter(_) => {
            let in_same_function = matches!(
              (
                identifier.get_parent("function_definition"),
                candidate.get_parent("function_definition"),
              ),
              (Some(f1), Some(f2)) if f1.id() == f2.id()
            );

            in_same_function
              && ["value", "function_parameters"]
                .contains(&candidate_parent_kind)
          }
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

            let containing_parameter = candidate
              .get_parent("parameter")
              .or_else(|| candidate.get_parent("variadic_parameter"));

            if let Some(containing_parameter) = containing_parameter {
              let containing_parameter_name = self.document.get_node_text(
                &containing_parameter.find("identifier").unwrap(),
              );

              let shadowed_by_preceding_parameter =
                candidate.get_recipe(self.document).is_some_and(|recipe| {
                  recipe
                    .parameters
                    .iter()
                    .take_while(|parameter| {
                      parameter.name != containing_parameter_name
                    })
                    .any(|parameter| parameter.name == name)
                });

              return !shadowed_by_preceding_parameter;
            }

            if let Some(recipe) = candidate.get_recipe(self.document) {
              return !recipe
                .parameters
                .iter()
                .any(|parameter| parameter.name == name);
            }

            if let Some(function) = candidate.get_function(self.document) {
              return !function
                .parameters
                .iter()
                .any(|parameter| parameter.value == name);
            }

            true
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
      "assignment" => self.document.find_variable(&name).map(Symbol::Variable),
      "function_call" => {
        self
          .document
          .find_function(&name)
          .map(Symbol::Function)
          .or_else(|| {
            BUILTINS
            .iter()
            .find(|builtin| matches!(
              builtin,
              Builtin::Function { name: function_name, aliases, .. }
                if name == *function_name || aliases.contains(&name.as_str())
            ))
            .map(Symbol::Builtin)
          })
      }
      "function_definition" => {
        self.document.find_function(&name).map(Symbol::Function)
      }
      "function_parameters" => {
        identifier.get_function(self.document).and_then(|function| {
          function
            .parameters
            .iter()
            .find(|parameter| parameter.value == name)
            .cloned()
            .map(Symbol::FunctionParameter)
        })
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
      "value" => {
        let containing_parameter = identifier
          .get_parent("parameter")
          .or_else(|| identifier.get_parent("variadic_parameter"));

        match containing_parameter {
          None => identifier
            .get_recipe(self.document)
            .and_then(|recipe| {
              recipe
                .parameters
                .iter()
                .find(|parameter| parameter.name == name)
                .cloned()
                .map(Symbol::Parameter)
            })
            .or_else(|| {
              identifier.get_function(self.document).and_then(|function| {
                function
                  .parameters
                  .iter()
                  .find(|parameter| parameter.value == name)
                  .cloned()
                  .map(Symbol::FunctionParameter)
              })
            })
            .or_else(|| {
              self.document.find_variable(&name).map(Symbol::Variable)
            })
            .or_else(|| builtin_constant(&name)),
          Some(containing_parameter) => {
            let containing_parameter_name = self
              .document
              .get_node_text(&containing_parameter.find("identifier")?);

            identifier
              .get_recipe(self.document)
              .and_then(|recipe| {
                recipe
                  .parameters
                  .iter()
                  .take_while(|parameter| {
                    parameter.name != containing_parameter_name
                  })
                  .find(|parameter| parameter.name == name)
                  .cloned()
                  .map(Symbol::Parameter)
              })
              .or_else(|| {
                self.document.find_variable(&name).map(Symbol::Variable)
              })
              .or_else(|| builtin_constant(&name))
          }
        }
      }
      _ => BUILTINS
        .iter()
        .find(|builtin| match builtin {
          Builtin::Attribute {
            name: attribute_name,
            ..
          } => parent_kind == "attribute" && name == *attribute_name,
          Builtin::Setting {
            name: setting_name, ..
          } => parent_kind == "setting" && name == *setting_name,
          _ => false,
        })
        .map(Symbol::Builtin),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  #[test]
  fn resolve_shadowed_parameter_default_definition() {
    let document = Document::from(indoc! {
      "
      a := 'foo'

      b a=a:
        echo {{ a }}
      "
    });

    let definition = Resolver::new(&document)
      .resolve_identifier_definition(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("parameter > value > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      definition,
      lsp::Location {
        uri: document.uri.clone(),
        range: lsp::Range::at(0, 0, 1, 0),
      }
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

    let definition = Resolver::new(&document)
      .resolve_identifier_definition(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("dependency_expression > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      definition,
      lsp::Location {
        uri: document.uri.clone(),
        range: lsp::Range::at(0, 0, 3, 0),
      }
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

    let definition = Resolver::new(&document)
      .resolve_identifier_definition(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("dependency > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      definition,
      lsp::Location {
        uri: document.uri.clone(),
        range: lsp::Range::at(0, 0, 3, 0),
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

    let definition = Resolver::new(&document)
      .resolve_identifier_definition(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("value > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      definition,
      lsp::Location {
        uri: document.uri.clone(),
        range: lsp::Range::at(0, 0, 1, 0),
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

    let definition = Resolver::new(&document)
      .resolve_identifier_definition(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("value > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      definition,
      lsp::Location {
        uri: document.uri.clone(),
        range: lsp::Range::at(0, 4, 0, 19),
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

    let definition = Resolver::new(&document)
      .resolve_identifier_definition(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("function_call > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      definition,
      lsp::Location {
        uri: document.uri.clone(),
        range: lsp::Range::at(1, 10, 1, 14),
      }
    );
  }

  #[test]
  fn resolve_self_definition() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    let definition = Resolver::new(&document)
      .resolve_identifier_definition(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("recipe_header > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      definition,
      lsp::Location {
        uri: document.uri.clone(),
        range: lsp::Range::at(0, 0, 2, 0),
      }
    );
  }

  #[test]
  fn resolve_user_function_definition() {
    let document = Document::from(indoc! {
      "
      foo(x) := x + \"!\"

      bar:
        echo {{ foo(\"baz\") }}
      "
    });

    let definition = Resolver::new(&document)
      .resolve_identifier_definition(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("function_call > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      definition,
      lsp::Location {
        uri: document.uri.clone(),
        range: lsp::Range::at(0, 0, 0, 3),
      }
    );
  }

  #[test]
  fn resolve_user_function_parameter_definition() {
    let document = Document::from(indoc! {
      "
      foo(x) := x
      "
    });

    let definition = Resolver::new(&document)
      .resolve_identifier_definition(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("value > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      definition,
      lsp::Location {
        uri: document.uri.clone(),
        range: lsp::Range::at(0, 4, 0, 5),
      }
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

    let hover = Resolver::new(&document)
      .resolve_identifier_hover(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("dependency_expression > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::PlainText,
        value: "foo:\n  echo foo".to_string(),
      })
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

    let hover = resolver
      .resolve_identifier_hover(&root.find("dependency > identifier").unwrap())
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
  fn resolve_recipe_hover_in_alias() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      alias f := foo
      "
    });

    let hover = Resolver::new(&document)
      .resolve_identifier_hover(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("alias > identifier[1]")
          .unwrap(),
      )
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

    let hover = Resolver::new(&document)
      .resolve_identifier_hover(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("value > identifier")
          .unwrap(),
      )
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

    let hover = Resolver::new(&document)
      .resolve_identifier_hover(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("value > identifier")
          .unwrap(),
      )
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

    let hover = Resolver::new(&document)
      .resolve_identifier_hover(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("value > identifier")
          .unwrap(),
      )
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

    let hover = Resolver::new(&document)
      .resolve_identifier_hover(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("value > identifier")
          .unwrap(),
      )
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

    let hover = Resolver::new(&document)
      .resolve_identifier_hover(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("value > identifier")
          .unwrap(),
      )
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

    let hover = Resolver::new(&document)
      .resolve_identifier_hover(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("function_call > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::Markdown,
        value: indoc! {
          "
          Instruction set architecture of the host machine.

          Returns one of: `aarch64`, `arm`, `asmjs`, `hexagon`, `mips`,
          `msp430`, `powerpc`, `powerpc64`, `s390x`, `sparc`, `wasm32`,
          `x86`, `x86_64`, or `xcore`.

          ```just
          system-info:
            @echo This is an {{arch()}} machine.
          ```
          "
        }
        .to_string(),
      })
    );
  }

  #[test]
  fn resolve_builtin_constant_hover() {
    let document = Document::from(indoc! {
      "
      foo:
        echo {{ RED }}
      "
    });

    let hover = Resolver::new(&document)
      .resolve_identifier_hover(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("value > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::Markdown,
        value: indoc! {
          "
          ANSI escape sequence for red foreground text: `\\e[31m`.

          Terminate styled output with `NORMAL` to reset.
          "
        }
        .to_string(),
      })
    );
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

    let hover = Resolver::new(&document)
      .resolve_identifier_hover(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("attribute > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::Markdown,
        value: indoc! {
          "
          Don't change directory before executing the recipe.

          Normally `just` runs recipes with the current directory set to
          the directory containing the `justfile`. With `[no-cd]`, the
          recipe runs with the current directory unchanged, so it can use
          paths relative to the invocation directory or operate on the
          user's current directory.

          ```just
          [no-cd]
          commit file:
            git add {{file}}
            git commit
          ```
          "
        }
        .to_string(),
      })
    );
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

    let hover = Resolver::new(&document)
      .resolve_identifier_hover(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("setting > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::Markdown,
        value: indoc! {
          "
          Export every top-level `just` variable as an environment
          variable.

          Equivalent to prefixing each assignment with `export`, so
          recipes and backticks see the variables as `$NAME` rather than
          needing `{{ name }}` interpolation.

          ```just
          set export

          a := \"hello\"

          @foo b:
            echo $a
            echo $b
          ```
          "
        }
        .to_string(),
      })
    );
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

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::Markdown,
        value: indoc! {
          "
          Instruction set architecture of the host machine.

          Returns one of: `aarch64`, `arm`, `asmjs`, `hexagon`, `mips`,
          `msp430`, `powerpc`, `powerpc64`, `s390x`, `sparc`, `wasm32`,
          `x86`, `x86_64`, or `xcore`.

          ```just
          system-info:
            @echo This is an {{arch()}} machine.
          ```
          "
        }
        .to_string(),
      })
    );
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

    let hover = Resolver::new(&document)
      .resolve_identifier_hover(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("value > identifier")
          .unwrap(),
      )
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

    let hover = Resolver::new(&document).resolve_identifier_hover(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("text")
        .unwrap(),
    );

    assert_eq!(hover, None);
  }

  #[test]
  fn resolve_hover_nonexistent_variable() {
    let document = Document::from(indoc! {
      "
      foo:
        echo {{ nonexistent }}
      "
    });

    let hover = Resolver::new(&document).resolve_identifier_hover(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("value > identifier")
        .unwrap(),
    );

    assert_eq!(hover, None);
  }

  #[test]
  fn resolve_user_function_hover() {
    let document = Document::from(indoc! {
      "
      foo(x) := x + \"!\"

      bar:
        echo {{ foo(\"baz\") }}
      "
    });

    let hover = Resolver::new(&document)
      .resolve_identifier_hover(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("function_call > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::PlainText,
        value: "foo(x) := x + \"!\"".to_string(),
      })
    );
  }

  #[test]
  fn resolve_user_function_parameter_hover() {
    let document = Document::from(indoc! {
      "
      foo(x) := x
      "
    });

    let hover = Resolver::new(&document)
      .resolve_identifier_hover(
        &document
          .tree
          .as_ref()
          .unwrap()
          .root_node()
          .find("value > identifier")
          .unwrap(),
      )
      .unwrap();

    assert_eq!(
      hover.contents,
      lsp::HoverContents::Markup(lsp::MarkupContent {
        kind: lsp::MarkupKind::PlainText,
        value: "x".to_string(),
      })
    );
  }

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

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("recipe_header > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(0, 0, 0, 3),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(3, 9, 3, 12),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(6, 13, 6, 16),
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

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("parameter > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(5, 4, 5, 7),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(6, 10, 6, 13),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(7, 10, 7, 13),
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

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("value > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(2, 4, 2, 7),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(3, 10, 3, 13),
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

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("value > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(0, 0, 0, 3),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(3, 10, 3, 13),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(3, 16, 3, 19),
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

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("assignment > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(0, 0, 0, 3),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(3, 10, 3, 13),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(10, 10, 10, 13),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(11, 10, 11, 13),
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

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("assignment > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(0, 0, 0, 1),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(2, 4, 2, 5),
        },
      ]
    );
  }

  #[test]
  fn resolve_variable_excludes_parameter_default_shadowed_by_preceding_parameter()
   {
    let document = Document::from(indoc! {
      "
      a := 'foo'

      bar a b=a:
        echo {{ b }}
      "
    });

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("assignment > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![lsp::Location {
        uri: document.uri.clone(),
        range: lsp::Range::at(0, 0, 0, 1),
      }]
    );
  }

  #[test]
  fn resolve_variable_references_in_assignment_value() {
    let document = Document::from(indoc! {
      "
      foo := 'x'
      bar := foo
      "
    });

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("assignment > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(0, 0, 0, 3),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(1, 7, 1, 10),
        },
      ]
    );
  }

  #[test]
  fn resolve_variable_references_in_user_function_body() {
    let document = Document::from(indoc! {
      "
      base := 'x'

      join(ext) := base + ext
      "
    });

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("assignment > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(0, 0, 0, 4),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(2, 13, 2, 17),
        },
      ]
    );
  }

  #[test]
  fn resolve_variable_excludes_user_function_parameter_shadow() {
    let document = Document::from(indoc! {
      "
      base := 'global'

      join(base) := base + '!'
      "
    });

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("assignment > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![lsp::Location {
        uri: document.uri.clone(),
        range: lsp::Range::at(0, 0, 0, 4),
      }]
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

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("dependency > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(0, 5, 0, 8),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(2, 0, 2, 3),
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

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("dependency_expression > expression > value > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(0, 0, 0, 1),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(3, 10, 3, 11),
        },
      ]
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

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("dependency_expression > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(0, 0, 0, 3),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(3, 6, 3, 9),
        },
      ]
    );
  }

  #[test]
  fn resolve_user_function_references() {
    let document = Document::from(indoc! {
      "
      foo(x) := x + \"!\"

      bar:
        echo {{ foo(\"a\") }}
        echo {{ foo(\"b\") }}
      "
    });

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("function_definition > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(0, 0, 0, 3),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(3, 10, 3, 13),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(4, 10, 4, 13),
        },
      ]
    );
  }

  #[test]
  fn resolve_user_function_parameter_references() {
    let document = Document::from(indoc! {
      "
      foo(x) := x + x

      bar(x) := x
      "
    });

    let references = Resolver::new(&document).resolve_identifier_references(
      &document
        .tree
        .as_ref()
        .unwrap()
        .root_node()
        .find("function_parameters > identifier")
        .unwrap(),
    );

    assert_eq!(
      references,
      vec![
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(0, 4, 0, 5),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(0, 10, 0, 11),
        },
        lsp::Location {
          uri: document.uri.clone(),
          range: lsp::Range::at(0, 14, 0, 15),
        },
      ]
    );
  }
}
