use super::*;

#[derive(Debug)]
pub struct Document {
  pub content: Rope,
  pub tree: Tree,
  pub uri: lsp::Url,
  pub version: i32,
}

impl Document {
  /// Returns the alias declarations in source order.
  ///
  /// Duplicate aliases are preserved. An alias is omitted if either side of
  /// its declaration is absent from the syntax tree.
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::Document;
  ///
  /// let document = Document::from("alias foo := bar\nbar:\n");
  ///
  /// let aliases = document.aliases();
  ///
  /// assert_eq!(aliases[0].name.value, "foo");
  /// assert_eq!(aliases[0].value.value, "bar");
  /// ```
  #[must_use]
  pub fn aliases(&self) -> Vec<Alias> {
    self
      .tree
      .root_node()
      .find_all("alias")
      .iter()
      .filter_map(|alias_node| {
        Some(Alias {
          attributes: self.get_attributes(alias_node),
          name: TextNode::from_node(
            &alias_node.child_by_field_name("left")?,
            self,
          ),
          value: TextNode::from_node(
            &alias_node.child_by_field_name("right")?,
            self,
          ),
          range: self.get_range(alias_node),
        })
      })
      .collect()
  }

  /// Applies the client's changes and reparses the syntax tree.
  ///
  /// Changes are applied in order, so each range refers to the contents after
  /// the preceding change. A change without a range replaces the entire
  /// document. Range columns are measured in UTF-16 code units.
  ///
  /// The document's version is set to the version supplied by the client.
  /// The contents, version, and tree edits are retained if parsing fails.
  /// Syntax errors in the contents are represented in the tree and do not
  /// cause this method to return an error.
  ///
  /// # Errors
  ///
  /// Returns an error if the just grammar cannot be loaded or tree-sitter
  /// fails to produce a syntax tree.
  ///
  /// # Panics
  ///
  /// Panics if a change's start position is after its end position after
  /// conversion to character offsets.
  ///
  /// # Example
  ///
  /// ```
  /// use {
  ///   just_lsp::Document,
  ///   tower_lsp::lsp_types::{
  ///     DidChangeTextDocumentParams, TextDocumentContentChangeEvent,
  ///     VersionedTextDocumentIdentifier,
  ///   },
  /// };
  ///
  /// let mut document = Document::from("foo:\n");
  ///
  /// document
  ///   .apply_change(DidChangeTextDocumentParams {
  ///     text_document: VersionedTextDocumentIdentifier {
  ///       uri: document.uri.clone(),
  ///       version: 2,
  ///     },
  ///     content_changes: vec![TextDocumentContentChangeEvent {
  ///       range: None,
  ///       range_length: None,
  ///       text: "bar:\n".into(),
  ///     }],
  ///   })
  ///   .unwrap();
  ///
  /// assert_eq!(document.content.to_string(), "bar:\n");
  /// assert_eq!(document.recipes()[0].name.value, "bar");
  /// assert_eq!(document.version, 2);
  /// ```
  pub fn apply_change(
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

      self.tree.edit(&edit.input_edit);
    }

    self.parse()?;

    Ok(())
  }

  /// Returns the attributes throughout the document in source order.
  ///
  /// Each name in an attribute list produces a separate [`Attribute`]. These
  /// attributes share the range of the whole list, but retain their own name
  /// and argument ranges. Attributes without a recognized target are
  /// included with a `None` target.
  ///
  /// To restrict the search to a subtree, use [`Document::get_attributes`].
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::Document;
  ///
  /// let document = Document::from("[private, group('foo')]\nbar:\n");
  ///
  /// let names = document
  ///   .attributes()
  ///   .into_iter()
  ///   .map(|attribute| attribute.name.value)
  ///   .collect::<Vec<_>>();
  ///
  /// assert_eq!(names, ["private", "group"]);
  /// ```
  #[must_use]
  pub fn attributes(&self) -> Vec<Attribute> {
    self
      .tree
      .root_node()
      .find_all("attribute")
      .into_iter()
      .flat_map(|node| self.get_attributes(&node))
      .collect()
  }

  /// Returns the document's contents formatted by `just --fmt`.
  ///
  /// This runs `just --fmt --unstable --quiet` on a temporary file, using the
  /// indentation in `config` when one is provided. If the URI can be converted
  /// to a local file path, the temporary file is created alongside the
  /// document so relative paths can be resolved. Otherwise, the system's
  /// temporary directory is used.
  ///
  /// The document's contents, syntax tree, and version are unchanged.
  ///
  /// # Errors
  ///
  /// Returns an error if the file path has no parent, the temporary file
  /// cannot be created, written, or read as UTF-8, or `just` cannot be run.
  /// A nonzero exit status from `just` is returned as [`Error::Format`] with
  /// the command's standard error output.
  ///
  /// # Example
  ///
  /// This requires `just` to be available on `PATH`.
  ///
  /// ```no_run
  /// use {
  ///   just_lsp::{Document, FormattingConfig},
  ///   tower_lsp::lsp_types::Url,
  /// };
  ///
  /// let uri = Url::parse("untitled:foo").unwrap();
  /// let document = Document::new("foo:= 'bar'\n", uri).unwrap();
  ///
  /// let formatted = document.format(&FormattingConfig::default()).unwrap();
  ///
  /// assert_eq!(formatted, "foo := 'bar'\n");
  /// ```
  pub fn format(&self, config: &FormattingConfig) -> Result<String> {
    let file = if let Ok(path) = self.uri.file_path() {
      tempfile::Builder::new()
        .prefix(".justfile-fmt-")
        .tempfile_in(
          path
            .parent()
            .ok_or_else(|| Error::Format("file path has no parent".into()))?,
        )?
    } else {
      tempfile::Builder::new()
        .prefix(".justfile-fmt-")
        .tempfile()?
    };

    let content = self.content.to_string();

    fs::write(&file, content.as_bytes())?;

    let mut command = process::Command::new("just");

    command.arg("--fmt").arg("--unstable").arg("--quiet");

    if let Some(indentation) = &config.indentation {
      command.arg("--indentation").arg(indentation);
    }

    let output = command.arg("--justfile").arg(file.path()).output()?;

    if !output.status.success() {
      return Err(Error::Format(format!(
        "just formatting failed: {}",
        String::from_utf8_lossy(&output.stderr)
      )));
    }

    Ok(fs::read_to_string(&file)?)
  }

  /// Returns the function calls throughout the document in source order.
  ///
  /// Nested calls are included, with an outer call preceding calls in its
  /// arguments. Arguments retain their source text and are not evaluated. Calls
  /// without an identifier in the syntax tree are omitted.
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::Document;
  ///
  /// let document = Document::from("foo := bar(baz('qux'))\n");
  ///
  /// let names = document
  ///   .function_calls()
  ///   .into_iter()
  ///   .map(|call| call.name.value)
  ///   .collect::<Vec<_>>();
  ///
  /// assert_eq!(names, ["bar", "baz"]);
  /// ```
  #[must_use]
  pub fn function_calls(&self) -> Vec<FunctionCall> {
    self
      .tree
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
              .map(|argument_node| TextNode::from_node(&argument_node, self))
              .collect::<Vec<_>>()
          })
          .unwrap_or_default();

        Some(FunctionCall {
          name: TextNode::from_node(&identifier_node, self),
          arguments,
          range: self.get_range(&function_call_node),
        })
      })
      .collect()
  }

  /// Returns the user-defined functions in source order.
  ///
  /// Each function includes its attributes, parameters, body text, and source
  /// range. The complete declaration text is trimmed of surrounding
  /// whitespace. Definitions without a name node are omitted.
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::Document;
  ///
  /// let document = Document::from("foo(bar) := bar + 'baz'\n");
  ///
  /// let functions = document.functions();
  ///
  /// assert_eq!(functions[0].name.value, "foo");
  /// assert_eq!(functions[0].parameters[0].value, "bar");
  /// assert_eq!(functions[0].body, "bar + 'baz'");
  /// ```
  #[must_use]
  pub fn functions(&self) -> Vec<Function> {
    self
      .tree
      .root_node()
      .find_all("function_definition")
      .iter()
      .filter_map(|function_node| {
        let name_node = function_node.child_by_field_name("name")?;

        let parameters = function_node
          .child_by_field_name("parameters")
          .map(|params_node| {
            params_node
              .find_all("^identifier")
              .iter()
              .map(|param_node| TextNode::from_node(param_node, self))
              .collect::<Vec<_>>()
          })
          .unwrap_or_default();

        let body = function_node
          .child_by_field_name("body")
          .map(|body_node| self.get_node_text(&body_node))
          .unwrap_or_default();

        Some(Function {
          attributes: self.get_attributes(function_node),
          name: TextNode::from_node(&name_node, self),
          parameters,
          body,
          content: self.get_node_text(function_node).trim().to_string(),
          range: self.get_range(function_node),
        })
      })
      .collect()
  }

  /// Returns the attributes in the subtree rooted at `node` in source order.
  ///
  /// If `node` is itself an attribute list, its attributes are included. Each
  /// name in a list produces a separate [`Attribute`] with the list's range.
  ///
  /// The target is inferred from `node`, or from its parent when `node` is an
  /// attribute list. All returned attributes use this target, which is `None`
  /// if the node kind is not a supported attribute target. To infer the target
  /// separately for each list in the document, use [`Document::attributes`].
  ///
  /// `node` must belong to this document's current syntax tree.
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::{AttributeTarget, Document, NodeExt};
  ///
  /// let document = Document::from("[private]\nfoo:\n");
  ///
  /// let root = document.tree.root_node();
  /// let recipe = root.find("recipe").unwrap();
  /// let attributes = document.get_attributes(&recipe);
  ///
  /// assert_eq!(attributes[0].name.value, "private");
  /// assert_eq!(attributes[0].target, Some(AttributeTarget::Recipe));
  /// ```
  #[must_use]
  pub fn get_attributes(&self, node: &Node) -> Vec<Attribute> {
    let target = if node.kind() == "attribute" {
      node.parent()
    } else {
      Some(*node)
    };

    let target =
      target.and_then(|node| AttributeTarget::try_from_kind(node.kind()));

    node
      .find_all("attribute")
      .into_iter()
      .flat_map(|attribute| {
        attribute
          .find_all("^identifier")
          .into_iter()
          .map(move |identifier| {
            let arguments = identifier
              .siblings()
              .take_while(|sibling| sibling.kind() != "identifier")
              .filter(|sibling| {
                sibling.start_byte() != sibling.end_byte()
                  && matches!(
                    sibling.kind(),
                    "string" | "expression" | "attribute_named_param"
                  )
              })
              .map(|argument| TextNode::from_node(&argument, self))
              .collect::<Vec<_>>();

            Attribute {
              name: TextNode::from_node(&identifier, self),
              arguments,
              target,
              range: self.get_range(&attribute),
            }
          })
          .collect::<Vec<_>>()
      })
      .collect()
  }

  /// Returns the function definition enclosing `node`.
  ///
  /// Only ancestors are searched; `node` itself is not considered. Returns
  /// `None` if there is no enclosing function or its definition cannot be
  /// extracted by [`Document::functions`].
  ///
  /// `node` must belong to this document's current syntax tree.
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::{Document, NodeExt};
  ///
  /// let document = Document::from("foo(bar) := bar\n");
  ///
  /// let root = document.tree.root_node();
  /// let name = root.find("identifier").unwrap();
  ///
  /// assert_eq!(document.get_function(&name), document.functions().pop());
  /// assert_eq!(document.get_function(&root), None);
  /// ```
  #[must_use]
  pub fn get_function(&self, node: &Node) -> Option<Function> {
    let range = self.get_range(&node.get_parent("function_definition")?);

    self
      .functions()
      .into_iter()
      .find(|function| function.range == range)
  }

  /// Returns a copy of the source text covered by `node`.
  ///
  /// Whitespace and delimiters within the node's range are preserved. `node`
  /// must belong to this document's current syntax tree.
  ///
  /// # Panics
  ///
  /// Panics if the node's byte range extends beyond the document's contents.
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::Document;
  ///
  /// let document = Document::from("foo:\n  bar\n");
  ///
  /// let root = document.tree.root_node();
  ///
  /// assert_eq!(document.get_node_text(&root), "foo:\n  bar\n");
  /// ```
  #[must_use]
  pub fn get_node_text(&self, node: &Node) -> String {
    self
      .content
      .slice(
        self.content.byte_to_char(node.start_byte())
          ..self.content.byte_to_char(node.end_byte()),
      )
      .to_string()
  }

  /// Returns the LSP range covered by `node`.
  ///
  /// Lines and columns are zero-based, and columns are measured in UTF-16
  /// code units. The end position is exclusive. `node` must belong to this
  /// document's current syntax tree.
  ///
  /// # Panics
  ///
  /// Panics if the node's byte range extends beyond the document's contents,
  /// or if a line or UTF-16 column cannot be represented as a `u32`.
  ///
  /// # Example
  ///
  /// This string contains a character that occupies two UTF-16 code units.
  ///
  /// ```
  /// use {
  ///   just_lsp::{Document, NodeExt},
  ///   tower_lsp::lsp_types::{Position, Range},
  /// };
  ///
  /// let document = Document::from("foo := '🧪'\n");
  ///
  /// let root = document.tree.root_node();
  /// let string = root.find("string").unwrap();
  ///
  /// assert_eq!(
  ///   document.get_range(&string),
  ///   Range::new(Position::new(0, 7), Position::new(0, 11)),
  /// );
  /// ```
  #[must_use]
  pub fn get_range(&self, node: &Node) -> lsp::Range {
    lsp::Range {
      start: self.content.byte_to_lsp_position(node.start_byte()),
      end: self.content.byte_to_lsp_position(node.end_byte()),
    }
  }

  /// Returns the recipe enclosing `node`.
  ///
  /// Only ancestors are searched; `node` itself is not considered. Returns
  /// `None` if there is no enclosing recipe or its declaration cannot be
  /// extracted by [`Document::recipes`].
  ///
  /// `node` must belong to this document's current syntax tree.
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::{Document, NodeExt};
  ///
  /// let document = Document::from("foo:\n  bar\n");
  ///
  /// let root = document.tree.root_node();
  /// let name = root.find("identifier").unwrap();
  ///
  /// assert_eq!(document.get_recipe(&name), document.recipes().pop());
  /// assert_eq!(document.get_recipe(&root), None);
  /// ```
  #[must_use]
  pub fn get_recipe(&self, node: &Node) -> Option<Recipe> {
    let range = self.get_range(&node.get_parent("recipe")?);

    self
      .recipes()
      .into_iter()
      .find(|recipe| recipe.range == range)
  }

  /// Returns the import declarations in source order.
  ///
  /// Paths retain their source spelling, including quotes and any string
  /// prefix. Optional imports are included. This method does not resolve
  /// paths or read imported files. Declarations without a path node are
  /// omitted.
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::Document;
  ///
  /// let document = Document::from("import? 'foo.just'\n");
  ///
  /// let imports = document.imports();
  ///
  /// assert_eq!(imports[0].path.value, "'foo.just'");
  /// assert!(imports[0].optional);
  /// ```
  #[must_use]
  pub fn imports(&self) -> Vec<Import> {
    self
      .tree
      .root_node()
      .find_all("import")
      .iter()
      .filter_map(|import_node| {
        let path_node = import_node.find("string")?;

        Some(Import {
          attributes: self.get_attributes(import_node),
          optional: import_node.find("^?").is_some(),
          path: TextNode::from_node(&path_node, self),
          range: self.get_range(import_node),
        })
      })
      .collect()
  }

  /// Returns the module declarations in source order.
  ///
  /// A module's path is `None` when no explicit path is given. Explicit paths
  /// retain their source spelling, including quotes and any string prefix.
  /// Optional modules are included. This method does not resolve paths or
  /// load modules. Declarations without a name node are omitted.
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::Document;
  ///
  /// let document = Document::from("mod? foo\n");
  ///
  /// let modules = document.modules();
  ///
  /// assert_eq!(modules[0].name.value, "foo");
  /// assert!(modules[0].optional);
  /// assert_eq!(modules[0].path, None);
  /// ```
  #[must_use]
  pub fn modules(&self) -> Vec<Module> {
    self
      .tree
      .root_node()
      .find_all("module")
      .iter()
      .filter_map(|module_node| {
        let name_node = module_node.child_by_field_name("name")?;

        let path = module_node
          .find("^string")
          .map(|path_node| TextNode::from_node(&path_node, self));

        Some(Module {
          attributes: self.get_attributes(module_node),
          name: TextNode::from_node(&name_node, self),
          optional: module_node.find("^?").is_some(),
          path,
          range: self.get_range(module_node),
        })
      })
      .collect()
  }

  /// Creates a document by parsing `source` and associating it with `uri`.
  ///
  /// The initial version is `0`. The URI identifies the document; its
  /// contents are taken entirely from `source`, without reading a file.
  ///
  /// Source containing syntax errors is accepted. Use
  /// `document.tree.root_node().has_error()` to check for those errors.
  ///
  /// # Errors
  ///
  /// Returns an error if the just grammar cannot be loaded or tree-sitter
  /// fails to produce a syntax tree.
  ///
  /// # Example
  ///
  /// ```
  /// use {just_lsp::Document, tower_lsp::lsp_types::Url};
  ///
  /// let uri = Url::parse("file:///foo.just").unwrap();
  /// let document = Document::new("foo :=\n", uri.clone()).unwrap();
  ///
  /// assert_eq!(document.content.to_string(), "foo :=\n");
  /// assert_eq!(document.uri, uri);
  /// assert_eq!(document.version, 0);
  /// assert!(document.tree.root_node().has_error());
  /// ```
  pub fn new(source: &str, uri: lsp::Url) -> Result<Self> {
    let content = Rope::from_str(source);

    let tree = Self::parse_tree(&content, None)?;

    Ok(Self {
      content,
      tree,
      uri,
      version: 0,
    })
  }

  /// Returns the smallest syntax tree node at the given LSP position.
  ///
  /// The line and column are zero-based, and the column is measured in UTF-16
  /// code units. Columns beyond the line's contents are clamped to the end of
  /// that line, before its line ending. Lines beyond the document are clamped
  /// to the end of the document. A position within a surrogate pair refers to
  /// the start of that character.
  ///
  /// The result can be an unnamed node, such as punctuation, or an enclosing
  /// node when the position falls between children.
  ///
  /// # Example
  ///
  /// ```
  /// use {just_lsp::Document, tower_lsp::lsp_types::Position};
  ///
  /// let document = Document::from("foo:\n");
  ///
  /// let node = document.node_at_position(Position::new(0, 1)).unwrap();
  ///
  /// assert_eq!(node.kind(), "identifier");
  /// assert_eq!(document.get_node_text(&node), "foo");
  /// ```
  #[must_use]
  pub fn node_at_position(&self, position: lsp::Position) -> Option<Node<'_>> {
    let byte = self
      .content
      .char_to_byte(self.content.lsp_position_to_char(position));

    self.tree.root_node().descendant_for_byte_range(byte, byte)
  }

  /// Reparses the current contents, reusing the existing syntax tree.
  ///
  /// If the contents have changed, the existing tree must first be updated
  /// with matching [`Tree::edit`] calls. [`Document::apply_change`] handles
  /// both the text and tree edits for changes received from the client.
  ///
  /// Syntax errors in the contents are represented in the new tree. The
  /// contents and version are unchanged, and the existing tree is replaced
  /// only when parsing succeeds.
  ///
  /// # Errors
  ///
  /// Returns an error if the just grammar cannot be loaded or tree-sitter
  /// fails to produce a syntax tree.
  ///
  /// # Example
  ///
  /// This applies an edit directly before reparsing the contents.
  ///
  /// ```
  /// use {
  ///   just_lsp::{Document, RopeExt},
  ///   tower_lsp::lsp_types::TextDocumentContentChangeEvent,
  /// };
  ///
  /// let mut document = Document::from("foo:\n");
  ///
  /// let change = TextDocumentContentChangeEvent {
  ///   range: None,
  ///   range_length: None,
  ///   text: "bar:\n".into(),
  /// };
  ///
  /// let edit = document.content.build_edit(&change);
  ///
  /// document.content.apply_edit(&edit);
  /// document.tree.edit(&edit.input_edit);
  ///
  /// document.parse().unwrap();
  ///
  /// assert_eq!(document.recipes()[0].name.value, "bar");
  /// assert_eq!(document.version, 1);
  /// ```
  pub fn parse(&mut self) -> Result {
    self.tree = Self::parse_tree(&self.content, Some(&self.tree))?;

    Ok(())
  }

  /// Parses `content` with the just grammar, optionally reusing `old_tree`.
  ///
  /// A reused tree must already have been edited to match `content`. Syntax
  /// errors are preserved in the returned tree.
  ///
  /// # Errors
  ///
  /// Returns an error if the grammar cannot be loaded or tree-sitter fails to
  /// produce a syntax tree.
  fn parse_tree(content: &Rope, old_tree: Option<&Tree>) -> Result<Tree> {
    let mut parser = Parser::new();

    // SAFETY: tree_sitter_just returns a static language definition.
    parser.set_language(&unsafe { tree_sitter_just() })?;

    parser
      .parse(content.to_string(), old_tree)
      .ok_or(Error::Parse)
  }

  /// Returns the recipe declarations in source order.
  ///
  /// Each recipe includes its attributes, parameters, dependencies, body,
  /// shebang, and source range. Dependencies retain their order and whether
  /// they occur before or after `&&`. They are not resolved to other recipes.
  /// Declarations without a name node are omitted.
  ///
  /// Body entries preserve indentation and omit blank lines. Newlines inside
  /// interpolations do not split an entry. The complete declaration text is
  /// trimmed of surrounding whitespace.
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::Document;
  ///
  /// let document = Document::from("foo bar: baz\n  echo {{bar}}\n");
  ///
  /// let recipes = document.recipes();
  ///
  /// assert_eq!(recipes[0].name.value, "foo");
  /// assert_eq!(recipes[0].parameters[0].name, "bar");
  /// assert_eq!(recipes[0].dependencies[0].name.value, "baz");
  /// assert_eq!(recipes[0].body[0].value, "  echo {{bar}}");
  /// ```
  #[must_use]
  pub fn recipes(&self) -> Vec<Recipe> {
    self
      .tree
      .root_node()
      .find_all("recipe")
      .iter()
      .filter_map(|recipe_node| {
        let name_node = recipe_node.find("recipe_header > identifier")?;

        let recipe_name = TextNode::from_node(&name_node, self);

        let body =
          recipe_node
            .find("^recipe_body")
            .map_or_else(Vec::new, |body_node| {
              let interpolations = body_node.find_all("interpolation");

              let offset =
                body_node.start_byte() - body_node.start_position().column;

              self
                .content
                .byte_slice(offset..body_node.end_byte())
                .bytes()
                .enumerate()
                .filter_map(|(index, byte)| {
                  (byte == b'\n').then_some(offset + index)
                })
                .chain(once(body_node.end_byte()))
                .filter(|end| {
                  !interpolations.iter().any(|interpolation| {
                    interpolation.start_byte() <= *end
                      && *end < interpolation.end_byte()
                  })
                })
                .scan(offset, |start, end| {
                  let range = *start..end;
                  *start = end + 1;
                  Some(range)
                })
                .filter_map(|range| {
                  let value =
                    self.content.byte_slice(range.clone()).to_string();

                  (!value.trim().is_empty()).then(|| TextNode {
                    value,
                    range: lsp::Range {
                      start: self.content.byte_to_lsp_position(range.start),
                      end: self.content.byte_to_lsp_position(range.end),
                    },
                  })
                })
                .collect()
            });

        let dependencies = recipe_node
          .find("recipe_header > dependencies")
          .map(|dependencies_node| {
            let mut dependencies = Vec::new();
            let mut phase = DependencyPhase::Prior;

            for index in 0..dependencies_node.child_count() {
              let Some(node) = dependencies_node.child(index) else {
                continue;
              };

              match node.kind() {
                "&&" => {
                  phase = DependencyPhase::Subsequent;
                }
                "dependency" => {
                  let dependency_node = node;

                  let Some(dependency_name_node) =
                    dependency_node.child_by_field_name("name").or_else(|| {
                      dependency_node
                        .find("dependency_expression")
                        .and_then(|node| node.child_by_field_name("name"))
                    })
                  else {
                    continue;
                  };

                  let arguments = dependency_node
                    .find("dependency_expression")
                    .map(|dependency_expression_node| {
                      let mut cursor = dependency_expression_node.walk();

                      dependency_expression_node
                        .named_children(&mut cursor)
                        .filter_map(|argument_node| {
                          match argument_node.kind() {
                            "expression" => Some(DependencyArgument {
                              value: self.get_node_text(&argument_node),
                              range: self.get_range(&argument_node),
                              starred: None,
                            }),
                            "starred_dependency_argument" => {
                              let value_node = argument_node
                                .child_by_field_name("argument")?;

                              Some(DependencyArgument {
                                value: self.get_node_text(&value_node),
                                range: self.get_range(&value_node),
                                starred: argument_node
                                  .child_by_field_name("star")
                                  .map(|node| self.get_range(&node)),
                              })
                            }
                            _ => None,
                          }
                        })
                        .collect()
                    })
                    .unwrap_or_default();

                  let mapped = dependency_node
                    .find("dependency_expression")
                    .and_then(|dependency_expression_node| {
                      dependency_expression_node
                        .child_by_field_name("map")
                        .map(|node| self.get_range(&node))
                    });

                  dependencies.push(Dependency {
                    name: TextNode::from_node(&dependency_name_node, self),
                    arguments,
                    mapped,
                    phase,
                    range: self.get_range(&dependency_node),
                  });
                }
                _ => {}
              }
            }

            dependencies
          })
          .unwrap_or_default();

        let parameters = recipe_node
          .find("recipe_header > parameters")
          .map_or_else(Vec::new, |parameters_node| {
            parameters_node
              .find_all("^parameter, ^variadic_parameter")
              .iter()
              .filter_map(|parameter_node| {
                Parameter::from_node(parameter_node, self)
              })
              .collect()
          });

        let shebang = recipe_node
          .find("recipe_body > shebang")
          .map(|shebang_node| TextNode::from_node(&shebang_node, self));

        Some(Recipe {
          name: recipe_name,
          attributes: self.get_attributes(recipe_node),
          body,
          dependencies,
          content: self.get_node_text(recipe_node).trim().to_string(),
          parameters,
          range: self.get_range(recipe_node),
          shebang,
        })
      })
      .collect()
  }

  /// Returns the setting declarations in source order.
  ///
  /// Values retain their source text. A boolean flag with no explicit value
  /// has an empty value and a [`SettingKind::Boolean`] kind set to `true`.
  /// Declarations that cannot be extracted by [`Setting::from_node`] are
  /// omitted.
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::{Document, SettingKind};
  ///
  /// let document = Document::from("set dotenv-load\n");
  ///
  /// let settings = document.settings();
  ///
  /// assert_eq!(settings[0].name.value, "dotenv-load");
  /// assert_eq!(settings[0].value.value, "");
  /// assert!(matches!(settings[0].kind, SettingKind::Boolean(true)));
  /// ```
  #[must_use]
  pub fn settings(&self) -> Vec<Setting> {
    self
      .tree
      .root_node()
      .find_all("setting")
      .iter()
      .filter_map(|setting_node| Setting::from_node(setting_node, self))
      .collect()
  }

  /// Returns the `unexport` declarations in source order.
  ///
  /// Each declaration includes its attributes, variable name, and source
  /// range. Declarations without a name node are omitted. This method does
  /// not modify the environment.
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::Document;
  ///
  /// let document = Document::from("unexport foo\n");
  ///
  /// let names = document
  ///   .unexports()
  ///   .into_iter()
  ///   .map(|unexport| unexport.name.value)
  ///   .collect::<Vec<_>>();
  ///
  /// assert_eq!(names, ["foo"]);
  /// ```
  #[must_use]
  pub fn unexports(&self) -> Vec<Unexport> {
    self
      .tree
      .root_node()
      .find_all("unexport")
      .iter()
      .filter_map(|unexport_node| {
        let name_node = unexport_node.child_by_field_name("name")?;

        Some(Unexport {
          attributes: self.get_attributes(unexport_node),
          name: TextNode::from_node(&name_node, self),
          range: self.get_range(unexport_node),
        })
      })
      .collect()
  }

  /// Returns the variable assignments in source order.
  ///
  /// Ordinary, exported, and eager assignments are included. Attributes on
  /// an `export` or `eager` declaration are attached to its variable, while
  /// the content and range cover the assignment itself. The content is
  /// trimmed of surrounding whitespace. Assignments without a name node
  /// are omitted.
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::Document;
  ///
  /// let document = Document::from("export foo := 'bar'\n");
  ///
  /// let variables = document.variables();
  ///
  /// assert_eq!(variables[0].name.value, "foo");
  /// assert!(variables[0].export);
  /// assert_eq!(variables[0].content, "foo := 'bar'");
  /// ```
  #[must_use]
  pub fn variables(&self) -> Vec<Variable> {
    self
      .tree
      .root_node()
      .find_all("assignment")
      .iter()
      .filter_map(|assignment_node| {
        let identifier_node = assignment_node.child_by_field_name("left")?;

        let attribute_node = assignment_node
          .parent()
          .filter(|parent| matches!(parent.kind(), "eager" | "export"))
          .unwrap_or(*assignment_node);

        Some(Variable {
          attributes: self.get_attributes(&attribute_node),
          name: TextNode::from_node(&identifier_node, self),
          export: identifier_node.get_parent("export").is_some(),
          content: self.get_node_text(assignment_node).trim().to_string(),
          range: self.get_range(assignment_node),
        })
      })
      .collect()
  }
}

impl From<&str> for Document {
  /// Creates a document with URI `file:///test.just` and version `1`.
  ///
  /// This is a convenience for tests and examples. Use [`Document::new`] to
  /// supply a URI and handle parser errors. Syntax errors in `value` are
  /// accepted and represented in the syntax tree.
  ///
  /// # Panics
  ///
  /// Panics if [`Document::new`] returns an error.
  fn from(value: &str) -> Self {
    Self {
      version: 1,
      ..Self::new(value, lsp::Url::parse("file:///test.just").unwrap()).unwrap()
    }
  }
}

impl TryFrom<lsp::DidOpenTextDocumentParams> for Document {
  type Error = Error;

  /// Creates a document from a client's `textDocument/didOpen` notification.
  ///
  /// The supplied text is parsed with the just grammar, and the URI and
  /// version are preserved. The language identifier is ignored.
  ///
  /// # Errors
  ///
  /// Returns an error if [`Document::new`] fails. Syntax errors in the text
  /// are accepted and represented in the syntax tree.
  fn try_from(params: lsp::DidOpenTextDocumentParams) -> Result<Self> {
    let lsp::TextDocumentItem {
      text, uri, version, ..
    } = params.text_document;

    Ok(Self {
      version,
      ..Self::new(&text, uri)?
    })
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*, indoc::indoc, parameter::VariadicType,
    pretty_assertions::assert_eq,
  };

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
        range: Some(lsp::Range::at(1, 7, 2, 0)),
        range_length: None,
        text: "\"bar\"".to_string(),
      }],
    };

    document.apply_change(change).unwrap();

    assert_ne!(document.content.to_string(), original_content);
    assert_eq!(document.content.to_string(), "foo:\n  echo \"bar\"");
    assert_eq!(document.recipes()[0].content, "foo:\n  echo \"bar\"");
    assert_eq!(document.version, 2);
    assert!(!document.tree.root_node().has_error());
  }

  #[test]
  fn apply_change_reparses_syntax_errors() {
    let mut document = Document::from("foo :=\n");

    assert!(document.tree.root_node().has_error());

    document
      .apply_change(lsp::DidChangeTextDocumentParams {
        text_document: lsp::VersionedTextDocumentIdentifier {
          uri: document.uri.clone(),
          version: 2,
        },
        content_changes: vec![lsp::TextDocumentContentChangeEvent {
          range: Some(lsp::Range::at(0, 6, 0, 6)),
          range_length: None,
          text: " 'bar'".into(),
        }],
      })
      .unwrap();

    assert!(!document.tree.root_node().has_error());
    assert_eq!(document.variables()[0].content, "foo := 'bar'");
  }

  #[test]
  fn create_document() {
    #[track_caller]
    fn case(source: &str, has_error: bool) {
      let uri = lsp::Url::parse("file:///foo.just").unwrap();

      let document = Document::new(source, uri.clone()).unwrap();

      assert_eq!(document.content.to_string(), source);
      assert_eq!(document.tree.root_node().has_error(), has_error);
      assert_eq!(document.uri, uri);
      assert_eq!(document.version, 0);
    }

    case("", false);
    case("foo:\n  echo foo\n", false);
    case("foo :=\n", true);
  }

  #[test]
  fn create_document_from_open_params() {
    let uri = lsp::Url::parse("file:///foo.just").unwrap();

    let document = Document::try_from(lsp::DidOpenTextDocumentParams {
      text_document: lsp::TextDocumentItem {
        uri: uri.clone(),
        language_id: "just".into(),
        version: 2,
        text: "foo:".into(),
      },
    })
    .unwrap();

    assert_eq!(document.recipes()[0].name.value, "foo");
    assert_eq!(document.uri, uri);
    assert_eq!(document.version, 2);
  }

  #[test]
  fn create_document_from_str() {
    let document = Document::from("foo:");

    assert_eq!(document.recipes()[0].name.value, "foo");
    assert_eq!(document.uri.as_str(), "file:///test.just");
    assert_eq!(document.version, 1);
  }

  #[test]
  fn eager_variable_with_attributes() {
    let document = Document::from(indoc! {
      "
      [private]
      eager foo := 'bar'
      "
    });

    assert_eq!(
      document.variables(),
      vec![Variable {
        attributes: vec![Attribute {
          arguments: vec![],
          name: TextNode {
            value: "private".into(),
            range: lsp::Range::at(0, 1, 0, 8),
          },
          range: lsp::Range::at(0, 0, 1, 0),
          target: Some(AttributeTarget::Assignment),
        }],
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(1, 6, 1, 9),
        },
        export: false,
        content: "foo := 'bar'".into(),
        range: lsp::Range::at(1, 6, 2, 0),
      }]
    );
  }

  #[test]
  fn function_no_parameters() {
    let document = Document::from(indoc! {
      "
      foo() := \"bar\"
      "
    });

    assert_eq!(
      document.functions(),
      vec![Function {
        attributes: vec![],
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(0, 0, 0, 3),
        },
        parameters: vec![],
        body: "\"bar\"".into(),
        content: "foo() := \"bar\"".into(),
        range: lsp::Range::at(0, 0, 1, 0),
      }],
    );
  }

  #[test]
  fn function_with_attributes() {
    let document = Document::from(indoc! {
      "
      [macos]
      foo() := 'bar'
      "
    });

    assert_eq!(
      document.functions(),
      vec![Function {
        attributes: vec![Attribute {
          arguments: vec![],
          name: TextNode {
            value: "macos".into(),
            range: lsp::Range::at(0, 1, 0, 6),
          },
          range: lsp::Range::at(0, 0, 1, 0),
          target: Some(AttributeTarget::Function),
        }],
        body: "'bar'".into(),
        content: "[macos]\nfoo() := 'bar'".into(),
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(1, 0, 1, 3),
        },
        parameters: vec![],
        range: lsp::Range::at(0, 0, 2, 0),
      }],
    );
  }

  #[test]
  fn get_alias_with_attributes() {
    let document = Document::from(indoc! {
      "
      [linux]
      alias foo := bar
      "
    });

    let aliases = document.aliases();

    assert_eq!(aliases.len(), 1);

    assert_eq!(
      aliases,
      vec![Alias {
        attributes: vec![Attribute {
          arguments: vec![],
          name: TextNode {
            value: "linux".into(),
            range: lsp::Range::at(0, 1, 0, 6),
          },
          range: lsp::Range::at(0, 0, 1, 0),
          target: Some(AttributeTarget::Alias),
        }],
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(1, 6, 1, 9),
        },
        range: lsp::Range::at(0, 0, 1, 16),
        value: TextNode {
          value: "bar".into(),
          range: lsp::Range::at(1, 13, 1, 16),
        },
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
        attributes: vec![],
        name: TextNode {
          value: "a1".into(),
          range: lsp::Range::at(0, 6, 0, 8)
        },
        value: TextNode {
          value: "tools::build".into(),
          range: lsp::Range::at(0, 12, 0, 24)
        },
        range: lsp::Range::at(0, 0, 0, 24)
      }]
    );
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
        attributes: vec![],
        name: TextNode {
          value: "shell".into(),
          range: lsp::Range::at(0, 4, 0, 9),
        },
        kind: SettingKind::Array,
        range: lsp::Range::at(0, 0, 1, 0),
        value: TextNode {
          value: "['foo']".into(),
          range: lsp::Range::at(0, 13, 0, 20),
        },
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
        attributes: vec![],
        name: TextNode {
          value: "a1".into(),
          range: lsp::Range::at(0, 6, 0, 8)
        },
        value: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(0, 12, 0, 15)
        },
        range: lsp::Range::at(0, 0, 0, 15)
      }]
    );
  }

  #[test]
  fn get_boolean_false_setting() {
    let document = Document::from(indoc! {
      "
      set foo := false
      "
    });

    let settings = document.settings();

    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings,
      vec![Setting {
        attributes: vec![],
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(0, 4, 0, 7),
        },
        kind: SettingKind::Boolean(false),
        range: lsp::Range::at(0, 0, 1, 0),
        value: TextNode {
          value: "false".into(),
          range: lsp::Range::at(0, 11, 0, 16),
        },
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
        attributes: vec![],
        name: TextNode {
          value: "export".into(),
          range: lsp::Range::at(0, 4, 0, 10),
        },
        kind: SettingKind::Boolean(true),
        range: lsp::Range::at(0, 0, 1, 0),
        value: TextNode {
          value: String::new(),
          range: lsp::Range::at(0, 10, 0, 10),
        },
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
        attributes: vec![],
        name: TextNode {
          value: "export".into(),
          range: lsp::Range::at(0, 4, 0, 10),
        },
        kind: SettingKind::Boolean(true),
        range: lsp::Range::at(0, 0, 1, 0),
        value: TextNode {
          value: "true".into(),
          range: lsp::Range::at(0, 14, 0, 18),
        },
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
          attributes: vec![],
          name: TextNode {
            value: "duplicate".into(),
            range: lsp::Range::at(0, 6, 0, 15)
          },
          value: TextNode {
            value: "foo".into(),
            range: lsp::Range::at(0, 19, 0, 22)
          },
          range: lsp::Range::at(0, 0, 0, 22)
        },
        Alias {
          attributes: vec![],
          name: TextNode {
            value: "duplicate".into(),
            range: lsp::Range::at(1, 6, 1, 15)
          },
          value: TextNode {
            value: "bar".into(),
            range: lsp::Range::at(1, 19, 1, 22)
          },
          range: lsp::Range::at(1, 0, 1, 22)
        }
      ]
    );
  }

  #[test]
  fn get_expression_setting() {
    let document = Document::from(indoc! {
      r#"
      set foo := "bar" / baz
      "#
    });

    let settings = document.settings();

    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings,
      vec![Setting {
        attributes: vec![],
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(0, 4, 0, 7),
        },
        kind: SettingKind::String,
        range: lsp::Range::at(0, 0, 1, 0),
        value: TextNode {
          value: "\"bar\" / baz".into(),
          range: lsp::Range::at(0, 11, 0, 22),
        },
      }]
    );
  }

  #[test]
  fn get_hyphenated_array_setting() {
    let document = Document::from(indoc! {
      r#"
      set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
      "#
    });

    let settings = document.settings();

    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings,
      vec![Setting {
        attributes: vec![],
        name: TextNode {
          value: "windows-shell".into(),
          range: lsp::Range::at(0, 4, 0, 17),
        },
        kind: SettingKind::Array,
        range: lsp::Range::at(0, 0, 1, 0),
        value: TextNode {
          value: "[\"powershell.exe\", \"-NoLogo\", \"-Command\"]".into(),
          range: lsp::Range::at(0, 21, 0, 62),
        },
      }]
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
          attributes: vec![],
          name: TextNode {
            value: "a1".into(),
            range: lsp::Range::at(0, 6, 0, 8),
          },
          value: TextNode {
            value: "foo".into(),
            range: lsp::Range::at(0, 12, 0, 15),
          },
          range: lsp::Range::at(0, 0, 0, 15),
        },
        Alias {
          attributes: vec![],
          name: TextNode {
            value: "a2".into(),
            range: lsp::Range::at(1, 6, 1, 8),
          },
          value: TextNode {
            value: "bar".into(),
            range: lsp::Range::at(1, 12, 1, 15),
          },
          range: lsp::Range::at(1, 0, 1, 15),
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
          attributes: vec![],
          name: TextNode {
            value: "export".into(),
            range: lsp::Range::at(0, 4, 0, 10),
          },
          kind: SettingKind::Boolean(true),
          range: lsp::Range::at(0, 0, 1, 0),
          value: TextNode {
            value: "true".into(),
            range: lsp::Range::at(0, 14, 0, 18),
          },
        },
        Setting {
          attributes: vec![],
          name: TextNode {
            value: "shell".into(),
            range: lsp::Range::at(1, 4, 1, 9),
          },
          kind: SettingKind::Array,
          range: lsp::Range::at(1, 0, 2, 0),
          value: TextNode {
            value: "['foo']".into(),
            range: lsp::Range::at(1, 13, 1, 20),
          },
        },
        Setting {
          attributes: vec![],
          name: TextNode {
            value: "bar".into(),
            range: lsp::Range::at(2, 4, 2, 7),
          },
          kind: SettingKind::String,
          range: lsp::Range::at(2, 0, 3, 0),
          value: TextNode {
            value: "'wow!'".into(),
            range: lsp::Range::at(2, 11, 2, 17),
          },
        }
      ]
    );
  }

  #[test]
  fn get_setting_with_attributes() {
    let document = Document::from(indoc! {
      "
      [group(\"foo\")]
      set bar := true
      "
    });

    let settings = document.settings();

    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings,
      vec![Setting {
        attributes: vec![Attribute {
          name: TextNode {
            value: "group".into(),
            range: lsp::Range::at(0, 1, 0, 6),
          },
          arguments: vec![TextNode {
            value: "\"foo\"".into(),
            range: lsp::Range::at(0, 7, 0, 12),
          }],
          target: Some(AttributeTarget::Setting),
          range: lsp::Range::at(0, 0, 1, 0),
        }],
        name: TextNode {
          value: "bar".into(),
          range: lsp::Range::at(1, 4, 1, 7),
        },
        kind: SettingKind::Boolean(true),
        range: lsp::Range::at(0, 0, 2, 0),
        value: TextNode {
          value: "true".into(),
          range: lsp::Range::at(1, 11, 1, 15),
        },
      }]
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
        attributes: vec![],
        name: TextNode {
          value: "bar".into(),
          range: lsp::Range::at(0, 4, 0, 7),
        },
        kind: SettingKind::String,
        range: lsp::Range::at(0, 0, 1, 0),
        value: TextNode {
          value: "'wow!'".into(),
          range: lsp::Range::at(0, 11, 0, 17),
        },
      }]
    );
  }

  #[test]
  fn get_string_setting_containing_walrus() {
    let document = Document::from(indoc! {
      r#"
      set foo := "bar := baz"
      "#
    });

    let settings = document.settings();

    assert_eq!(settings.len(), 1);

    assert_eq!(
      settings,
      vec![Setting {
        attributes: vec![],
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(0, 4, 0, 7),
        },
        kind: SettingKind::String,
        range: lsp::Range::at(0, 0, 1, 0),
        value: TextNode {
          value: "\"bar := baz\"".into(),
          range: lsp::Range::at(0, 11, 0, 23),
        },
      }]
    );
  }

  #[test]
  fn get_unexports() {
    let document = Document::from(indoc! {
      "
      [windows]
      unexport FOO
      "
    });

    let unexports = document.unexports();

    assert_eq!(unexports.len(), 1);

    assert_eq!(
      unexports,
      vec![Unexport {
        attributes: vec![Attribute {
          arguments: vec![],
          name: TextNode {
            value: "windows".into(),
            range: lsp::Range::at(0, 1, 0, 8),
          },
          range: lsp::Range::at(0, 0, 1, 0),
          target: Some(AttributeTarget::Unexport),
        }],
        name: TextNode {
          value: "FOO".into(),
          range: lsp::Range::at(1, 9, 1, 12),
        },
        range: lsp::Range::at(0, 0, 2, 0),
      }]
    );
  }

  #[test]
  fn get_variable_with_attributes() {
    let document = Document::from(indoc! {
      "
      [windows]
      foo := 'bar'
      "
    });

    let variables = document.variables();

    assert_eq!(variables.len(), 1);

    assert_eq!(
      variables,
      vec![Variable {
        attributes: vec![Attribute {
          arguments: vec![],
          name: TextNode {
            value: "windows".into(),
            range: lsp::Range::at(0, 1, 0, 8),
          },
          range: lsp::Range::at(0, 0, 1, 0),
          target: Some(AttributeTarget::Assignment),
        }],
        content: "[windows]\nfoo := 'bar'".into(),
        export: false,
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(1, 0, 1, 3),
        },
        range: lsp::Range::at(0, 0, 2, 0),
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
          attributes: vec![],
          name: TextNode {
            value: "tmpdir".into(),
            range: lsp::Range::at(0, 0, 0, 6),
          },
          export: false,
          content: "tmpdir  := `mktemp -d`".into(),
          range: lsp::Range::at(0, 0, 1, 0),
        },
        Variable {
          attributes: vec![],
          name: TextNode {
            value: "version".into(),
            range: lsp::Range::at(1, 0, 1, 7),
          },
          export: false,
          content: "version := \"0.2.7\"".into(),
          range: lsp::Range::at(1, 0, 2, 0),
        },
        Variable {
          attributes: vec![],
          name: TextNode {
            value: "tardir".into(),
            range: lsp::Range::at(2, 0, 2, 6),
          },
          export: false,
          content: "tardir  := tmpdir / \"awesomesauce-\" + version".into(),
          range: lsp::Range::at(2, 0, 3, 0),
        },
        Variable {
          attributes: vec![],
          name: TextNode {
            value: "tarball".into(),
            range: lsp::Range::at(3, 0, 3, 7),
          },
          export: false,
          content: "tarball := tardir + \".tar.gz\"".into(),
          range: lsp::Range::at(3, 0, 4, 0),
        },
        Variable {
          attributes: vec![],
          name: TextNode {
            value: "config".into(),
            range: lsp::Range::at(4, 0, 4, 6),
          },
          export: false,
          content: "config  := quote(config_dir() / \".project-config\")"
            .into(),
          range: lsp::Range::at(4, 0, 5, 0),
        },
        Variable {
          attributes: vec![],
          name: TextNode {
            value: "EDITOR".into(),
            range: lsp::Range::at(5, 7, 5, 13),
          },
          export: true,
          content: "EDITOR := 'nvim'".into(),
          range: lsp::Range::at(5, 7, 6, 0),
        },
      ]
    );
  }

  #[test]
  fn imports() {
    let document = Document::from(indoc! {
      "
      [linux]
      import 'foo/bar.just'

      a: b
        @echo A
      "
    });

    assert_eq!(
      document.imports(),
      vec![Import {
        attributes: vec![Attribute {
          arguments: vec![],
          name: TextNode {
            value: "linux".into(),
            range: lsp::Range::at(0, 1, 0, 6),
          },
          range: lsp::Range::at(0, 0, 1, 0),
          target: Some(AttributeTarget::Import),
        }],
        optional: false,
        path: TextNode {
          value: "'foo/bar.just'".into(),
          range: lsp::Range::at(1, 7, 1, 21),
        },
        range: lsp::Range::at(0, 0, 1, 21),
      }]
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

      [function_attr]
      function() := \"function\"

      [import_attr]
      import \"foo.just\"

      [unexport_attr]
      unexport FOO
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
            range: lsp::Range::at(0, 1, 0, 8),
          },
          range: lsp::Range::at(0, 0, 1, 0),
          target: Some(AttributeTarget::Recipe),
        },
        Attribute {
          arguments: vec![TextNode {
            value: "\"desc\"".into(),
            range: lsp::Range::at(0, 23, 0, 29),
          }],
          name: TextNode {
            value: "description".into(),
            range: lsp::Range::at(0, 10, 0, 21),
          },
          range: lsp::Range::at(0, 0, 1, 0),
          target: Some(AttributeTarget::Recipe),
        },
        Attribute {
          arguments: vec![],
          name: TextNode {
            value: "alias_attr".into(),
            range: lsp::Range::at(4, 1, 4, 11),
          },
          range: lsp::Range::at(4, 0, 5, 0),
          target: Some(AttributeTarget::Alias),
        },
        Attribute {
          arguments: vec![TextNode {
            value: "\"value\"".into(),
            range: lsp::Range::at(7, 10, 7, 17),
          }],
          name: TextNode {
            value: "var_attr".into(),
            range: lsp::Range::at(7, 1, 7, 9),
          },
          range: lsp::Range::at(7, 0, 8, 0),
          target: Some(AttributeTarget::Assignment),
        },
        Attribute {
          arguments: vec![],
          name: TextNode {
            value: "export_attr".into(),
            range: lsp::Range::at(10, 1, 10, 12),
          },
          range: lsp::Range::at(10, 0, 11, 0),
          target: Some(AttributeTarget::Assignment),
        },
        Attribute {
          arguments: vec![],
          name: TextNode {
            value: "module_attr".into(),
            range: lsp::Range::at(13, 1, 13, 12),
          },
          range: lsp::Range::at(13, 0, 14, 0),
          target: Some(AttributeTarget::Module),
        },
        Attribute {
          arguments: vec![],
          name: TextNode {
            value: "function_attr".into(),
            range: lsp::Range::at(16, 1, 16, 14),
          },
          range: lsp::Range::at(16, 0, 17, 0),
          target: Some(AttributeTarget::Function),
        },
        Attribute {
          arguments: vec![],
          name: TextNode {
            value: "import_attr".into(),
            range: lsp::Range::at(19, 1, 19, 12),
          },
          range: lsp::Range::at(19, 0, 20, 0),
          target: Some(AttributeTarget::Import),
        },
        Attribute {
          arguments: vec![],
          name: TextNode {
            value: "unexport_attr".into(),
            range: lsp::Range::at(22, 1, 22, 14),
          },
          range: lsp::Range::at(22, 0, 23, 0),
          target: Some(AttributeTarget::Unexport),
        },
      ],
    );
  }

  #[test]
  fn list_document_attributes_without_target() {
    let document = Document::from(indoc! {
      "
      [private]
      "
    });

    assert_eq!(
      document.attributes(),
      vec![Attribute {
        arguments: vec![],
        name: TextNode {
          value: "private".into(),
          range: lsp::Range::at(0, 1, 0, 8),
        },
        range: lsp::Range::at(0, 0, 1, 0),
        target: None,
      }]
    );
  }

  #[test]
  fn list_function_calls() {
    let document = Document::from(indoc! {
      "
      foo:
        echo {{arch()}}
        echo {{env_var(\"HOME\", \"fallback\")}}
        echo {{show(['foo', 'bar'])}}
      "
    });

    let calls = document.function_calls();

    assert_eq!(
      calls,
      vec![
        FunctionCall {
          arguments: vec![],
          name: TextNode {
            value: "arch".into(),
            range: lsp::Range::at(1, 9, 1, 13),
          },
          range: lsp::Range::at(1, 9, 1, 15),
        },
        FunctionCall {
          arguments: vec![
            TextNode {
              value: "\"HOME\"".into(),
              range: lsp::Range::at(2, 17, 2, 23),
            },
            TextNode {
              value: "\"fallback\"".into(),
              range: lsp::Range::at(2, 25, 2, 35),
            },
          ],
          name: TextNode {
            value: "env_var".into(),
            range: lsp::Range::at(2, 9, 2, 16),
          },
          range: lsp::Range::at(2, 9, 2, 36),
        },
        FunctionCall {
          arguments: vec![TextNode {
            value: "['foo', 'bar']".into(),
            range: lsp::Range::at(3, 14, 3, 28),
          }],
          name: TextNode {
            value: "show".into(),
            range: lsp::Range::at(3, 9, 3, 13),
          },
          range: lsp::Range::at(3, 9, 3, 29),
        },
      ],
    );
  }

  #[test]
  fn list_functions() {
    let document = Document::from(indoc! {
      "
      hello(name) := f\"Hello, \" + name

      greet(a, b) := hello(a) + \" and \" + hello(b)
      "
    });

    assert_eq!(
      document.functions(),
      vec![
        Function {
          attributes: vec![],
          name: TextNode {
            value: "hello".into(),
            range: lsp::Range::at(0, 0, 0, 5),
          },
          parameters: vec![TextNode {
            value: "name".into(),
            range: lsp::Range::at(0, 6, 0, 10),
          }],
          body: "f\"Hello, \" + name".into(),
          content: "hello(name) := f\"Hello, \" + name".into(),
          range: lsp::Range::at(0, 0, 1, 0),
        },
        Function {
          attributes: vec![],
          name: TextNode {
            value: "greet".into(),
            range: lsp::Range::at(2, 0, 2, 5),
          },
          parameters: vec![
            TextNode {
              value: "a".into(),
              range: lsp::Range::at(2, 6, 2, 7),
            },
            TextNode {
              value: "b".into(),
              range: lsp::Range::at(2, 9, 2, 10),
            },
          ],
          body: "hello(a) + \" and \" + hello(b)".into(),
          content: "greet(a, b) := hello(a) + \" and \" + hello(b)".into(),
          range: lsp::Range::at(2, 0, 3, 0),
        },
      ],
    );
  }

  #[test]
  fn module_path_ignores_attribute_strings() {
    #[track_caller]
    fn case(source: &str, expected: Option<&TextNode>) {
      assert_eq!(Document::from(source).modules()[0].path.as_ref(), expected);
    }

    case(
      "[doc: 'foo']\nmod bar 'baz.just'",
      Some(&TextNode {
        value: "'baz.just'".into(),
        range: lsp::Range::at(1, 8, 1, 18),
      }),
    );

    case("[doc: 'foo']\nmod bar", None);
  }

  #[test]
  fn module_with_path() {
    let document = Document::from(indoc! {
      r#"
      [private]
      mod foo "./utils.just"
      "#
    });

    assert_eq!(
      document.modules(),
      vec![Module {
        attributes: vec![Attribute {
          arguments: vec![],
          name: TextNode {
            value: "private".into(),
            range: lsp::Range::at(0, 1, 0, 8),
          },
          range: lsp::Range::at(0, 0, 1, 0),
          target: Some(AttributeTarget::Module),
        }],
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(1, 4, 1, 7),
        },
        optional: false,
        path: Some(TextNode {
          value: "\"./utils.just\"".into(),
          range: lsp::Range::at(1, 8, 1, 22),
        }),
        range: lsp::Range::at(0, 0, 1, 22),
      }]
    );
  }

  #[test]
  fn module_without_path() {
    let document = Document::from(indoc! {
      "
      mod foo
      "
    });

    assert_eq!(
      document.modules(),
      vec![Module {
        attributes: vec![],
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(0, 4, 0, 7),
        },
        optional: false,
        path: None,
        range: lsp::Range::at(0, 0, 0, 7),
      }]
    );
  }

  #[test]
  fn multiple_imports() {
    let document = Document::from(indoc! {
      "
      import 'foo.just'
      import? 'bar.just'
      "
    });

    assert_eq!(
      document.imports(),
      vec![
        Import {
          attributes: vec![],
          optional: false,
          path: TextNode {
            value: "'foo.just'".into(),
            range: lsp::Range::at(0, 7, 0, 17),
          },
          range: lsp::Range::at(0, 0, 0, 17),
        },
        Import {
          attributes: vec![],
          optional: true,
          path: TextNode {
            value: "'bar.just'".into(),
            range: lsp::Range::at(1, 8, 1, 18),
          },
          range: lsp::Range::at(1, 0, 1, 18),
        },
      ]
    );
  }

  #[test]
  fn multiple_modules() {
    let document = Document::from(indoc! {
      r#"
      mod foo
      mod? bar "bar.just"
      "#
    });

    assert_eq!(
      document.modules(),
      vec![
        Module {
          attributes: vec![],
          name: TextNode {
            value: "foo".into(),
            range: lsp::Range::at(0, 4, 0, 7),
          },
          optional: false,
          path: None,
          range: lsp::Range::at(0, 0, 0, 7),
        },
        Module {
          attributes: vec![],
          name: TextNode {
            value: "bar".into(),
            range: lsp::Range::at(1, 5, 1, 8),
          },
          optional: true,
          path: Some(TextNode {
            value: "\"bar.just\"".into(),
            range: lsp::Range::at(1, 9, 1, 19),
          }),
          range: lsp::Range::at(1, 0, 1, 19),
        },
      ]
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
      document.recipes(),
      vec![
        Recipe {
          name: TextNode {
            value: "foo".into(),
            range: lsp::Range::at(0, 0, 0, 3)
          },
          attributes: vec![],
          dependencies: vec![],
          parameters: vec![],
          content: "foo:\n  echo \"foo\"".into(),
          body: vec![TextNode {
            value: "  echo \"foo\"".into(),
            range: lsp::Range::at(1, 0, 1, 12),
          }],
          range: lsp::Range::at(0, 0, 3, 0),
          shebang: None,
        },
        Recipe {
          name: TextNode {
            value: "bar".into(),
            range: lsp::Range::at(3, 0, 3, 3)
          },
          attributes: vec![],
          dependencies: vec![],
          parameters: vec![],
          content: "bar:\n  echo \"bar\"".into(),
          body: vec![TextNode {
            value: "  echo \"bar\"".into(),
            range: lsp::Range::at(4, 0, 4, 12),
          }],
          range: lsp::Range::at(3, 0, 5, 0),
          shebang: None,
        }
      ]
    );
  }

  #[test]
  fn node_at_position() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
      "
    });

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
    let document = Document::from(indoc! {
      "
      foo:
        echo \"a🧪b\"
      "
    });

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
  fn optional_import() {
    #[track_caller]
    fn case(source: &str, expected: bool) {
      assert_eq!(Document::from(source).imports()[0].optional, expected);
    }

    case("import 'foo.just'", false);
    case("import? 'foo.just'", true);
    case("import 'foo?bar.just'", false);
    case("import? 'foo?bar.just'", true);
  }

  #[test]
  fn optional_module() {
    #[track_caller]
    fn case(source: &str, expected: bool) {
      assert_eq!(Document::from(source).modules()[0].optional, expected);
    }

    case("mod foo", false);
    case("mod? foo", true);
    case("mod foo 'foo?bar.just'", false);
    case("mod? foo 'foo?bar.just'", true);
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
        attributes: vec![Attribute {
          arguments: vec![],
          name: TextNode {
            value: "private".into(),
            range: lsp::Range::at(0, 1, 0, 8),
          },
          range: lsp::Range::at(0, 0, 1, 0),
          target: Some(AttributeTarget::Assignment),
        }],
        name: TextNode {
          value: "PATH".into(),
          range: lsp::Range::at(1, 7, 1, 11),
        },
        export: true,
        content: "PATH := '/usr/local/bin'".into(),
        range: lsp::Range::at(1, 7, 2, 0),
      }]
    );
  }

  #[test]
  fn recipe_body_comments() {
    let document = Document::from(indoc! {
      "
      foo:
        # bar
        baz
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0].body,
      vec![
        TextNode {
          value: "  # bar".into(),
          range: lsp::Range::at(1, 0, 1, 7),
        },
        TextNode {
          value: "  baz".into(),
          range: lsp::Range::at(2, 0, 2, 5),
        },
      ],
    );
  }

  #[test]
  fn recipe_body_continuation() {
    let document = Document::from(indoc! {
      "
      foo:
        {{'bar'}}\\
      \t{{'baz'}}
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0].body,
      vec![
        TextNode {
          value: "  {{'bar'}}\\".into(),
          range: lsp::Range::at(1, 0, 1, 12),
        },
        TextNode {
          value: "\t{{'baz'}}".into(),
          range: lsp::Range::at(2, 0, 2, 10),
        },
      ],
    );
  }

  #[test]
  fn recipe_body_crlf() {
    let document = Document::from("[private]\r\nfoo:\r\n\tbar");

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0].body,
      vec![TextNode {
        value: "\tbar".into(),
        range: lsp::Range::at(2, 0, 2, 4),
      }],
    );
  }

  #[test]
  fn recipe_body_empty() {
    let document = Document::from(indoc! {
      "
      foo:
      bar:
        baz
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(document.recipes()[0].body, vec![]);
  }

  #[test]
  fn recipe_body_multiline_header() {
    let document = Document::from(indoc! {
      "
      foo: \\
      \tbar
        baz
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0].body,
      vec![TextNode {
        value: "  baz".into(),
        range: lsp::Range::at(2, 0, 2, 5),
      }],
    );
  }

  #[test]
  fn recipe_body_multiline_interpolation() {
    let document = Document::from(indoc! {
      "
      foo:
        {{'
      bar
          baz
      '}}
        qux
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0].body,
      vec![
        TextNode {
          value: "  {{'\nbar\n    baz\n'}}".into(),
          range: lsp::Range::at(1, 0, 4, 3),
        },
        TextNode {
          value: "  qux".into(),
          range: lsp::Range::at(5, 0, 5, 5),
        },
      ],
    );
  }

  #[test]
  fn recipe_body_multiline_parameter() {
    let document = Document::from(indoc! {
      "
      foo bar='''
      \tbaz
          qux
      ''':
        bar
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0].body,
      vec![TextNode {
        value: "  bar".into(),
        range: lsp::Range::at(4, 0, 4, 5),
      }],
    );
  }

  #[test]
  fn recipe_body_recipe_boundary() {
    let document = Document::from(indoc! {
      "
      foo:
        bar
      baz:
      \tqux
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0].body,
      vec![TextNode {
        value: "  bar".into(),
        range: lsp::Range::at(1, 0, 1, 5),
      }],
    );
  }

  #[test]
  fn recipe_body_shebang() {
    let document = Document::from(indoc! {
      "
      foo:
        #!/bin/sh
        bar
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0].body,
      vec![
        TextNode {
          value: "  #!/bin/sh".into(),
          range: lsp::Range::at(1, 0, 1, 11),
        },
        TextNode {
          value: "  bar".into(),
          range: lsp::Range::at(2, 0, 2, 5),
        },
      ],
    );
  }

  #[test]
  fn recipe_body_shebang_comment() {
    let document = Document::from(indoc! {
      "
      foo:
        bar
          #!/bin/sh
        qux
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0].body,
      vec![
        TextNode {
          value: "  bar".into(),
          range: lsp::Range::at(1, 0, 1, 5),
        },
        TextNode {
          value: "    #!/bin/sh".into(),
          range: lsp::Range::at(2, 0, 2, 13),
        },
        TextNode {
          value: "  qux".into(),
          range: lsp::Range::at(3, 0, 3, 5),
        },
      ],
    );
  }

  #[test]
  fn recipe_body_whitespace() {
    let document = Document::from(indoc! {
      "
      foo:
        bar

       \t
        baz
      "
    });

    assert!(!document.tree.root_node().has_error());

    assert_eq!(
      document.recipes()[0].body,
      vec![
        TextNode {
          value: "  bar".into(),
          range: lsp::Range::at(1, 0, 1, 5),
        },
        TextNode {
          value: "  baz".into(),
          range: lsp::Range::at(4, 0, 4, 5),
        },
      ],
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

    let recipe = &document.recipes()[0];

    assert_eq!(recipe.attributes.len(), 3);

    assert_eq!(
      recipe.attributes,
      vec![
        Attribute {
          name: TextNode {
            value: "private".into(),
            range: lsp::Range::at(0, 1, 0, 8),
          },
          arguments: vec![],
          target: Some(AttributeTarget::Recipe),
          range: lsp::Range::at(0, 0, 1, 0),
        },
        Attribute {
          name: TextNode {
            value: "description".into(),
            range: lsp::Range::at(1, 1, 1, 12),
          },
          arguments: vec![TextNode {
            value: "\"This is a test recipe\"".into(),
            range: lsp::Range::at(1, 14, 1, 37),
          }],
          target: Some(AttributeTarget::Recipe),
          range: lsp::Range::at(1, 0, 2, 0),
        },
        Attribute {
          name: TextNode {
            value: "tags".into(),
            range: lsp::Range::at(2, 1, 2, 5),
          },
          arguments: vec![
            TextNode {
              value: "\"test\"".into(),
              range: lsp::Range::at(2, 6, 2, 12),
            },
            TextNode {
              value: "\"example\"".into(),
              range: lsp::Range::at(2, 14, 2, 23),
            }
          ],
          target: Some(AttributeTarget::Recipe),
          range: lsp::Range::at(2, 0, 3, 0),
        }
      ]
    );
  }

  #[test]
  fn recipe_with_complex_default_parameter() {
    let document = Document::from(indoc! {
      r#"
      foo triple=(arch + "-unknown-unknown"):
      "#
    });

    assert_eq!(
      document.recipes()[0].parameters,
      vec![Parameter {
        name: "triple".into(),
        kind: ParameterKind::Normal,
        export: false,
        default_value: Some("(arch + \"-unknown-unknown\")".into()),
        content: "triple=(arch + \"-unknown-unknown\")".into(),
        range: lsp::Range::at(0, 4, 0, 38),
      }],
    );
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
      document.recipes()[0],
      Recipe {
        name: TextNode {
          value: "baz".into(),
          range: lsp::Range::at(0, 0, 0, 3)
        },
        attributes: vec![],
        dependencies: vec![],
        parameters: vec![
          Parameter {
            name: "first".into(),
            kind: ParameterKind::Normal,
            export: false,
            default_value: None,
            content: "first".into(),
            range: lsp::Range::at(0, 4, 0, 9),
          },
          Parameter {
            name: "second".into(),
            kind: ParameterKind::Normal,
            export: false,
            default_value: Some("\"default\"".into()),
            content: "second=\"default\"".into(),
            range: lsp::Range::at(0, 10, 0, 26),
          }
        ],
        content:
          "baz first second=\"default\":\n  echo \"{{first}} {{second}}\""
            .into(),
        body: vec![TextNode {
          value: "  echo \"{{first}} {{second}}\"".into(),
          range: lsp::Range::at(1, 0, 1, 29),
        }],
        range: lsp::Range::at(0, 0, 2, 0),
        shebang: None,
      }
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
      document.recipes()[1],
      Recipe {
        name: TextNode {
          value: "bar".into(),
          range: lsp::Range::at(3, 0, 3, 3)
        },
        attributes: vec![],
        dependencies: vec![Dependency {
          name: TextNode {
            value: "foo".into(),
            range: lsp::Range::at(3, 5, 3, 8),
          },
          arguments: vec![],
          mapped: None,
          phase: DependencyPhase::Prior,
          range: lsp::Range::at(3, 5, 3, 8),
        }],
        parameters: vec![],
        content: "bar: foo\n  echo \"bar\"".into(),
        body: vec![TextNode {
          value: "  echo \"bar\"".into(),
          range: lsp::Range::at(4, 0, 4, 12),
        }],
        range: lsp::Range::at(3, 0, 5, 0),
        shebang: None,
      }
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
      document.recipes()[1],
      Recipe {
        name: TextNode {
          value: "bar".into(),
          range: lsp::Range::at(3, 0, 3, 3)
        },
        attributes: vec![],
        dependencies: vec![Dependency {
          name: TextNode {
            value: "foo".into(),
            range: lsp::Range::at(3, 6, 3, 9),
          },
          arguments: vec![
            DependencyArgument {
              value: "'value1'".into(),
              range: lsp::Range::at(3, 10, 3, 18),
              starred: None,
            },
            DependencyArgument {
              value: "'value2'".into(),
              range: lsp::Range::at(3, 19, 3, 27),
              starred: None,
            }
          ],
          mapped: None,
          phase: DependencyPhase::Prior,
          range: lsp::Range::at(3, 5, 3, 28),
        }],
        parameters: vec![],
        content: "bar: (foo 'value1' 'value2')\n  echo \"bar\"".into(),
        body: vec![TextNode {
          value: "  echo \"bar\"".into(),
          range: lsp::Range::at(4, 0, 4, 12),
        }],
        range: lsp::Range::at(3, 0, 5, 0),
        shebang: None,
      }
    );
  }

  #[test]
  fn recipe_with_exported_variadic_parameter() {
    #[track_caller]
    fn case(source: &str, expected: VariadicType) {
      let parameter = &Document::from(source).recipes()[0].parameters[0];

      assert!(parameter.export);
      assert_eq!(parameter.kind, ParameterKind::Variadic(expected));
    }

    case("foo +$args:\n", VariadicType::OneOrMore);
    case("foo *$args:\n", VariadicType::ZeroOrMore);
  }

  #[test]
  fn recipe_with_invalid_parameters() {
    #[track_caller]
    fn case(source: &str) {
      assert_eq!(Document::from(source).recipes()[0].parameters, vec![]);
    }

    case("foo $:\n");
    case("foo +:\n");
    case("foo *:\n");
  }

  #[test]
  fn recipe_with_mapped_dependency_arguments() {
    let document = Document::from(indoc! {
      "
      bar arg *args:
        echo \"{{arg}} {{args}}\"

      foo args: *(bar args *args)
        echo \"foo\"
      "
    });

    assert_eq!(
      document.recipes()[1],
      Recipe {
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(3, 0, 3, 3)
        },
        attributes: vec![],
        dependencies: vec![Dependency {
          name: TextNode {
            value: "bar".into(),
            range: lsp::Range::at(3, 12, 3, 15),
          },
          arguments: vec![
            DependencyArgument {
              value: "args".into(),
              range: lsp::Range::at(3, 16, 3, 20),
              starred: None,
            },
            DependencyArgument {
              value: "args".into(),
              range: lsp::Range::at(3, 22, 3, 26),
              starred: Some(lsp::Range::at(3, 21, 3, 22)),
            }
          ],
          mapped: Some(lsp::Range::at(3, 10, 3, 11)),
          phase: DependencyPhase::Prior,
          range: lsp::Range::at(3, 10, 3, 27),
        }],
        parameters: vec![Parameter {
          name: "args".into(),
          kind: ParameterKind::Normal,
          export: false,
          default_value: None,
          content: "args".into(),
          range: lsp::Range::at(3, 4, 3, 8),
        }],
        content: "foo args: *(bar args *args)\n  echo \"foo\"".into(),
        body: vec![TextNode {
          value: "  echo \"foo\"".into(),
          range: lsp::Range::at(4, 0, 4, 12),
        }],
        range: lsp::Range::at(3, 0, 5, 0),
        shebang: None,
      }
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
      document.recipes()[2],
      Recipe {
        name: TextNode {
          value: "baz".into(),
          range: lsp::Range::at(6, 0, 6, 3)
        },
        attributes: vec![],
        dependencies: vec![Dependency {
          name: TextNode {
            value: "tools::foo".into(),
            range: lsp::Range::at(6, 5, 6, 15),
          },
          arguments: vec![],
          mapped: None,
          phase: DependencyPhase::Prior,
          range: lsp::Range::at(6, 5, 6, 15),
        }],
        parameters: vec![],
        content: "baz: tools::foo\n  echo \"baz\"".into(),
        body: vec![TextNode {
          value: "  echo \"baz\"".into(),
          range: lsp::Range::at(7, 0, 7, 12),
        }],
        range: lsp::Range::at(6, 0, 8, 0),
        shebang: None,
      }
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
      document.recipes()[2],
      Recipe {
        name: TextNode {
          value: "baz".into(),
          range: lsp::Range::at(6, 0, 6, 3)
        },
        attributes: vec![],
        dependencies: vec![
          Dependency {
            name: TextNode {
              value: "foo".into(),
              range: lsp::Range::at(6, 5, 6, 8),
            },
            arguments: vec![],
            mapped: None,
            phase: DependencyPhase::Prior,
            range: lsp::Range::at(6, 5, 6, 8),
          },
          Dependency {
            name: TextNode {
              value: "bar".into(),
              range: lsp::Range::at(6, 9, 6, 12),
            },
            arguments: vec![],
            mapped: None,
            phase: DependencyPhase::Prior,
            range: lsp::Range::at(6, 9, 6, 12),
          }
        ],
        parameters: vec![],
        content: "baz: foo bar\n  echo \"baz\"".into(),
        body: vec![TextNode {
          value: "  echo \"baz\"".into(),
          range: lsp::Range::at(7, 0, 7, 12),
        }],
        range: lsp::Range::at(6, 0, 8, 0),
        shebang: None,
      }
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
      document.recipes()[0],
      Recipe {
        name: TextNode {
          value: "bar".into(),
          range: lsp::Range::at(0, 0, 0, 3)
        },
        attributes: vec![],
        dependencies: vec![],
        parameters: vec![
          Parameter {
            name: "target".into(),
            kind: ParameterKind::Normal,
            export: false,
            default_value: None,
            content: "target".into(),
            range: lsp::Range::at(0, 4, 0, 10),
          },
          Parameter {
            name: "lol".into(),
            kind: ParameterKind::Normal,
            export: true,
            default_value: None,
            content: "$lol".into(),
            range: lsp::Range::at(0, 11, 0, 15),
          }
        ],
        content: "bar target $lol:\n  echo \"Building {{target}}\"".into(),
        body: vec![TextNode {
          value: "  echo \"Building {{target}}\"".into(),
          range: lsp::Range::at(1, 0, 1, 28),
        }],
        range: lsp::Range::at(0, 0, 2, 0),
        shebang: None,
      }
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

    let recipe = &document.recipes()[0];

    assert_eq!(
      recipe.shebang,
      Some(TextNode {
        value: "#!/usr/bin/env bash".into(),
        range: lsp::Range::at(1, 2, 1, 21),
      })
    );
  }

  #[test]
  fn recipe_with_subsequent_dependency() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      bar:
        echo \"bar\"

      baz: foo && bar
        echo \"baz\"
      "
    });

    assert_eq!(
      document.recipes()[2]
        .dependencies
        .iter()
        .map(|dependency| dependency.phase)
        .collect::<Vec<_>>(),
      vec![DependencyPhase::Prior, DependencyPhase::Subsequent],
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
      document.recipes()[0],
      Recipe {
        name: TextNode {
          value: "baz".into(),
          range: lsp::Range::at(0, 0, 0, 3)
        },
        attributes: vec![],
        dependencies: vec![],
        parameters: vec![
          Parameter {
            name: "first".into(),
            kind: ParameterKind::Normal,
            export: false,
            default_value: None,
            content: "first".into(),
            range: lsp::Range::at(0, 4, 0, 9),
          },
          Parameter {
            name: "second".into(),
            kind: ParameterKind::Variadic(VariadicType::OneOrMore),
            export: false,
            default_value: Some("\"default\"".into()),
            content: "+second=\"default\"".into(),
            range: lsp::Range::at(0, 10, 0, 27),
          }
        ],
        content:
          "baz first +second=\"default\":\n  echo \"{{first}} {{second}}\""
            .into(),
        body: vec![TextNode {
          value: "  echo \"{{first}} {{second}}\"".into(),
          range: lsp::Range::at(1, 0, 1, 29),
        }],
        range: lsp::Range::at(0, 0, 2, 0),
        shebang: None,
      }
    );
  }

  #[test]
  fn recipe_with_zero_or_more_variadic_parameter() {
    let document = Document::from(indoc! {
      "
      foo *FLAGS:
      "
    });

    assert_eq!(
      document.recipes()[0].parameters,
      vec![Parameter {
        name: "FLAGS".into(),
        kind: ParameterKind::Variadic(VariadicType::ZeroOrMore),
        export: false,
        default_value: None,
        content: "*FLAGS".into(),
        range: lsp::Range::at(0, 4, 0, 10),
      }],
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
      document.recipes()[0],
      Recipe {
        name: TextNode {
          value: "foo".into(),
          range: lsp::Range::at(0, 0, 0, 3)
        },
        attributes: vec![],
        dependencies: vec![],
        parameters: vec![],
        content: "foo:\n  echo \"foo\"".into(),
        body: vec![TextNode {
          value: "  echo \"foo\"".into(),
          range: lsp::Range::at(1, 0, 1, 12),
        }],
        range: lsp::Range::at(0, 0, 2, 0),
        shebang: None,
      }
    );
  }
}
