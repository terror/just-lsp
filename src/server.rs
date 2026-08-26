use super::*;

pub(crate) struct Server(Arc<Inner>);

impl Server {
  pub(crate) fn capabilities() -> lsp::ServerCapabilities {
    lsp::ServerCapabilities {
      completion_provider: Some(lsp::CompletionOptions {
        ..Default::default()
      }),
      code_action_provider: Some(lsp::CodeActionProviderCapability::Simple(
        true,
      )),
      code_lens_provider: Some(lsp::CodeLensOptions {
        resolve_provider: Some(false),
      }),
      definition_provider: Some(lsp::OneOf::Left(true)),
      document_symbol_provider: Some(lsp::OneOf::Left(true)),
      document_formatting_provider: Some(lsp::OneOf::Left(true)),
      document_link_provider: Some(lsp::DocumentLinkOptions {
        resolve_provider: Some(false),
        work_done_progress_options: lsp::WorkDoneProgressOptions::default(),
      }),
      document_highlight_provider: Some(lsp::OneOf::Left(true)),
      execute_command_provider: Some(lsp::ExecuteCommandOptions {
        commands: Command::all(),
        ..Default::default()
      }),
      folding_range_provider: Some(
        lsp::FoldingRangeProviderCapability::Simple(true),
      ),
      hover_provider: Some(lsp::HoverProviderCapability::Simple(true)),
      references_provider: Some(lsp::OneOf::Left(true)),
      rename_provider: Some(lsp::OneOf::Right(lsp::RenameOptions {
        prepare_provider: Some(true),
        work_done_progress_options: lsp::WorkDoneProgressOptions::default(),
      })),
      semantic_tokens_provider: Some(
        lsp::SemanticTokensServerCapabilities::SemanticTokensOptions(
          lsp::SemanticTokensOptions {
            legend: tokenizer::Tokenizer::legend().clone(),
            full: Some(lsp::SemanticTokensFullOptions::Bool(true)),
            range: None,
            ..Default::default()
          },
        ),
      ),
      text_document_sync: Some(lsp::TextDocumentSyncCapability::Options(
        lsp::TextDocumentSyncOptions {
          open_close: Some(true),
          change: Some(lsp::TextDocumentSyncKind::INCREMENTAL),
          will_save: None,
          will_save_wait_until: None,
          save: Some(
            lsp::SaveOptions {
              include_text: Some(false),
            }
            .into(),
          ),
        },
      )),
      ..Default::default()
    }
  }

  pub(crate) fn new(client: Client) -> Self {
    Self(Arc::new(Inner::new(client)))
  }

  pub(crate) async fn run() -> Result {
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) = LspService::new(Server::new);

    tower_lsp::Server::new(stdin, stdout, socket)
      .serve(service)
      .await;

    Ok(())
  }
}

impl Debug for Server {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.debug_struct("Server").finish()
  }
}

#[tower_lsp::async_trait]
impl LanguageServer for Server {
  async fn code_action(
    &self,
    params: lsp::CodeActionParams,
  ) -> Result<Option<lsp::CodeActionResponse>, jsonrpc::Error> {
    self.0.code_action(params).await
  }

  async fn code_lens(
    &self,
    params: lsp::CodeLensParams,
  ) -> Result<Option<Vec<lsp::CodeLens>>, jsonrpc::Error> {
    self.0.code_lens(params).await
  }

  async fn completion(
    &self,
    params: lsp::CompletionParams,
  ) -> Result<Option<lsp::CompletionResponse>, jsonrpc::Error> {
    self.0.completion(params).await
  }

  async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
    if let Err(error) = self.0.did_change(params).await {
      self
        .0
        .client
        .log_message(lsp::MessageType::ERROR, error)
        .await;
    }
  }

  async fn did_close(&self, params: lsp::DidCloseTextDocumentParams) {
    self.0.did_close(params).await;
  }

  async fn did_open(&self, params: lsp::DidOpenTextDocumentParams) {
    if let Err(error) = self.0.did_open(params).await {
      self
        .0
        .client
        .log_message(lsp::MessageType::ERROR, error)
        .await;
    }
  }

  async fn document_highlight(
    &self,
    params: lsp::DocumentHighlightParams,
  ) -> Result<Option<Vec<lsp::DocumentHighlight>>, jsonrpc::Error> {
    self.0.document_highlight(params).await
  }

  async fn document_link(
    &self,
    params: lsp::DocumentLinkParams,
  ) -> Result<Option<Vec<lsp::DocumentLink>>, jsonrpc::Error> {
    self.0.document_link(params).await
  }

  async fn document_symbol(
    &self,
    params: lsp::DocumentSymbolParams,
  ) -> Result<Option<lsp::DocumentSymbolResponse>, jsonrpc::Error> {
    self.0.document_symbol(params).await
  }

  async fn execute_command(
    &self,
    params: lsp::ExecuteCommandParams,
  ) -> Result<Option<serde_json::Value>, jsonrpc::Error> {
    self.0.execute_command(params).await
  }

  async fn folding_range(
    &self,
    params: lsp::FoldingRangeParams,
  ) -> Result<Option<Vec<lsp::FoldingRange>>, jsonrpc::Error> {
    self.0.folding_range(params).await
  }

  async fn formatting(
    &self,
    params: lsp::DocumentFormattingParams,
  ) -> Result<Option<Vec<lsp::TextEdit>>, jsonrpc::Error> {
    self.0.formatting(params).await
  }

  async fn goto_definition(
    &self,
    params: lsp::GotoDefinitionParams,
  ) -> Result<Option<lsp::GotoDefinitionResponse>, jsonrpc::Error> {
    self.0.goto_definition(params).await
  }

  async fn hover(
    &self,
    params: lsp::HoverParams,
  ) -> Result<Option<lsp::Hover>, jsonrpc::Error> {
    self.0.hover(params).await
  }

  async fn initialize(
    &self,
    params: lsp::InitializeParams,
  ) -> Result<lsp::InitializeResult, jsonrpc::Error> {
    self.0.initialize(params).await
  }

  async fn initialized(&self, params: lsp::InitializedParams) {
    self.0.initialized(params).await;
  }

  async fn prepare_rename(
    &self,
    params: lsp::TextDocumentPositionParams,
  ) -> Result<Option<lsp::PrepareRenameResponse>, jsonrpc::Error> {
    self.0.prepare_rename(params).await
  }

  async fn references(
    &self,
    params: lsp::ReferenceParams,
  ) -> Result<Option<Vec<lsp::Location>>, jsonrpc::Error> {
    self.0.references(params).await
  }

  async fn rename(
    &self,
    params: lsp::RenameParams,
  ) -> Result<Option<lsp::WorkspaceEdit>, jsonrpc::Error> {
    self.0.rename(params).await
  }

  async fn semantic_tokens_full(
    &self,
    params: lsp::SemanticTokensParams,
  ) -> Result<Option<lsp::SemanticTokensResult>, jsonrpc::Error> {
    self.0.semantic_tokens_full(params).await
  }

  #[allow(clippy::unused_async)]
  async fn shutdown(&self) -> Result<(), jsonrpc::Error> {
    Inner::shutdown().await
  }
}

pub(crate) struct Inner {
  client: Client,
  config: RwLock<Config>,
  executor: Executor,
  initialized: AtomicBool,
  workspace: RwLock<Workspace>,
}

impl Inner {
  async fn code_action(
    &self,
    params: lsp::CodeActionParams,
  ) -> Result<Option<lsp::CodeActionResponse>, jsonrpc::Error> {
    fn json<T: Serialize>(value: T) -> Result<Value, jsonrpc::Error> {
      serde_json::to_value(value).map_err(|_| jsonrpc::Error::parse_error())
    }

    let config = self.config.read().await;

    let workspace = self.workspace.read().await;

    let Some(document) =
      workspace.documents.get_open(&params.text_document.uri)
    else {
      return Ok(None);
    };

    let mut actions = Vec::new();

    for recipe in document.recipes() {
      let title = recipe.name.value.clone();

      let parameters = recipe
        .parameters
        .into_iter()
        .map(ParameterJson::from)
        .collect::<Vec<_>>();

      actions.push(lsp::CodeActionOrCommand::CodeAction(lsp::CodeAction {
        title: title.clone(),
        kind: Some(lsp::CodeActionKind::SOURCE),
        command: Some(lsp::Command {
          title,
          command: Command::RunRecipe.to_string(),
          arguments: Some(vec![
            json(&recipe.name.value)?,
            json(&params.text_document.uri)?,
            json(parameters)?,
          ]),
        }),
        ..Default::default()
      }));
    }

    let imported_documents = workspace
      .projects
      .get(&params.text_document.uri)
      .into_iter()
      .flat_map(|project| project.imported_documents(&workspace.documents));

    let diagnostics = Analyzer {
      config: Some(&config),
      document,
      imported_documents: imported_documents.collect(),
    }
    .analyze();

    actions.extend(
      Quickfixer {
        diagnostics: &diagnostics,
        parameters: &params,
      }
      .collect(),
    );

    Ok(Some(actions))
  }

  async fn code_lens(
    &self,
    params: lsp::CodeLensParams,
  ) -> Result<Option<Vec<lsp::CodeLens>>, jsonrpc::Error> {
    let uri = &params.text_document.uri;

    let workspace = self.workspace.read().await;

    if let Some(document) = workspace.documents.get_open(uri) {
      let mut lenses = Vec::new();

      for recipe in document.recipes() {
        let parameters = recipe
          .parameters
          .into_iter()
          .map(ParameterJson::from)
          .collect::<Vec<ParameterJson>>();

        let recipe_name = serde_json::to_value(&recipe.name.value)
          .map_err(|_| jsonrpc::Error::parse_error())?;

        let uri = serde_json::to_value(uri)
          .map_err(|_| jsonrpc::Error::parse_error())?;

        let parameters = serde_json::to_value(parameters)
          .map_err(|_| jsonrpc::Error::parse_error())?;

        lenses.push(lsp::CodeLens {
          range: recipe.name.range,
          command: Some(lsp::Command {
            title: "Run".into(),
            command: Command::RunRecipe.to_string(),
            arguments: Some(vec![recipe_name, uri, parameters]),
          }),
          data: None,
        });
      }

      return Ok(Some(lenses));
    }

    Ok(None)
  }

  async fn completion(
    &self,
    params: lsp::CompletionParams,
  ) -> Result<Option<lsp::CompletionResponse>, jsonrpc::Error> {
    let uri = params.text_document_position.text_document.uri;

    let workspace = self.workspace.read().await;

    if let Some(document) = workspace.documents.get_open(&uri) {
      let mut completion_items = Vec::new();

      let recipes = document.recipes();

      for recipe in recipes {
        completion_items.push(lsp::CompletionItem {
          label: recipe.name.value.clone(),
          kind: Some(lsp::CompletionItemKind::FUNCTION),
          documentation: Some(lsp::Documentation::MarkupContent(
            lsp::MarkupContent {
              kind: lsp::MarkupKind::PlainText,
              value: recipe.content,
            },
          )),
          insert_text: Some(recipe.name.value),
          insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
          ..Default::default()
        });
      }

      let variables = document.variables();

      for variable in variables {
        completion_items.push(lsp::CompletionItem {
          label: variable.name.value.clone(),
          kind: Some(lsp::CompletionItemKind::VARIABLE),
          documentation: Some(lsp::Documentation::MarkupContent(
            lsp::MarkupContent {
              kind: lsp::MarkupKind::PlainText,
              value: variable.content,
            },
          )),
          insert_text: Some(variable.name.value),
          insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
          ..Default::default()
        });
      }

      for function in document.functions() {
        let parameters = function
          .parameters
          .iter()
          .map(|parameter| parameter.value.as_str())
          .collect::<Vec<_>>()
          .join(", ");

        completion_items.push(lsp::CompletionItem {
          label: format!("{}({})", function.name.value, parameters),
          kind: Some(lsp::CompletionItemKind::FUNCTION),
          documentation: Some(lsp::Documentation::MarkupContent(
            lsp::MarkupContent {
              kind: lsp::MarkupKind::PlainText,
              value: function.content.clone(),
            },
          )),
          filter_text: Some(function.name.value.clone()),
          insert_text: Some(function.name.value),
          insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
          ..Default::default()
        });
      }

      for builtin in BUILTINS {
        completion_items.extend(builtin.completion_items());
      }

      return Ok(Some(lsp::CompletionResponse::Array(completion_items)));
    }

    Ok(None)
  }

  async fn did_change(
    &self,
    params: lsp::DidChangeTextDocumentParams,
  ) -> Result {
    let uri = params.text_document.uri.clone();

    let roots = {
      let mut workspace = self.workspace.write().await;

      if !workspace.documents.is_open(&uri) {
        return Ok(());
      }

      let roots = workspace.affected_roots(&uri);

      workspace.documents.change(params)?;
      workspace.load_projects(roots.iter().cloned())?;

      roots
    };

    for root in roots {
      self.publish_diagnostics(&root).await;
    }

    Ok(())
  }

  async fn did_close(&self, params: lsp::DidCloseTextDocumentParams) {
    let uri = params.text_document.uri.clone();

    let roots = {
      let mut workspace = self.workspace.write().await;
      let mut roots = workspace.affected_roots(&uri);

      let closed = workspace.documents.close(&params);

      workspace.projects.remove(&uri);
      roots.remove(&uri);

      if !closed {
        return;
      }

      if let Err(error) = workspace.load_projects(roots.iter().cloned()) {
        warn!(%error, "failed to rebuild affected projects");
      }

      roots
    };

    self.client.publish_diagnostics(uri, vec![], None).await;

    for root in roots {
      self.publish_diagnostics(&root).await;
    }
  }

  async fn did_open(&self, params: lsp::DidOpenTextDocumentParams) -> Result {
    let uri = params.text_document.uri.clone();

    let roots = {
      let mut workspace = self.workspace.write().await;
      let mut roots = workspace.affected_roots(&uri);

      roots.insert(uri.clone());

      workspace.documents.open(params)?;
      workspace.load_projects(roots.iter().cloned())?;

      roots
    };

    for root in roots {
      self.publish_diagnostics(&root).await;
    }

    Ok(())
  }

  async fn document_highlight(
    &self,
    params: lsp::DocumentHighlightParams,
  ) -> Result<Option<Vec<lsp::DocumentHighlight>>, jsonrpc::Error> {
    let uri = params.text_document_position_params.text_document.uri;

    let position = params.text_document_position_params.position;

    let workspace = self.workspace.read().await;

    Ok(workspace.documents.get_open(&uri).and_then(|document| {
      let resolver = Resolver::new(document);

      document
        .node_at_position(position)
        .filter(|node| node.kind() == "identifier")
        .map(|identifier| {
          resolver
            .resolve_identifier_references(&identifier)
            .into_iter()
            .map(|location| lsp::DocumentHighlight {
              range: location.range,
              kind: Some(lsp::DocumentHighlightKind::TEXT),
            })
            .collect()
        })
    }))
  }

  async fn document_link(
    &self,
    params: lsp::DocumentLinkParams,
  ) -> Result<Option<Vec<lsp::DocumentLink>>, jsonrpc::Error> {
    let uri = &params.text_document.uri;

    let workspace = self.workspace.read().await;

    let Some(document) = workspace.documents.get_open(uri) else {
      return Ok(None);
    };

    let mut links = Vec::new();

    if let Some(project) = workspace.projects.get(uri) {
      for dependency in project.dependencies(uri) {
        if let ProjectDependencyTarget::Resolved(target) = &dependency.target {
          links.push(lsp::DocumentLink {
            range: dependency.location,
            target: Some(target.clone()),
            tooltip: target
              .to_file_path()
              .ok()
              .map(|path| path.display().to_string()),
            data: None,
          });
        }
      }
    }

    for module in document.modules() {
      let range = module.path.as_ref().map_or(module.name.range, |p| p.range);

      if let Some(path) = module.resolve(uri)
        && let Ok(target) = lsp::Url::from_file_path(&path)
      {
        links.push(lsp::DocumentLink {
          range,
          target: Some(target),
          tooltip: Some(path.display().to_string()),
          data: None,
        });
      }
    }

    Ok(Some(links))
  }

  async fn document_symbol(
    &self,
    params: lsp::DocumentSymbolParams,
  ) -> Result<Option<lsp::DocumentSymbolResponse>, jsonrpc::Error> {
    let uri = &params.text_document.uri;

    let workspace = self.workspace.read().await;

    if let Some(document) = workspace.documents.get_open(uri) {
      let mut symbols = Vec::new();

      for recipe in document.recipes() {
        #[allow(deprecated)]
        symbols.push(lsp::DocumentSymbol {
          name: recipe.name.value,
          detail: None,
          kind: lsp::SymbolKind::FUNCTION,
          tags: None,
          deprecated: None,
          range: recipe.range,
          selection_range: recipe.name.range,
          children: None,
        });
      }

      for alias in document.aliases() {
        #[allow(deprecated)]
        symbols.push(lsp::DocumentSymbol {
          name: alias.name.value,
          detail: Some(format!("alias for {}", alias.value.value)),
          kind: lsp::SymbolKind::FUNCTION,
          tags: None,
          deprecated: None,
          range: alias.range,
          selection_range: alias.name.range,
          children: None,
        });
      }

      for variable in document.variables() {
        #[allow(deprecated)]
        symbols.push(lsp::DocumentSymbol {
          name: variable.name.value,
          detail: None,
          kind: lsp::SymbolKind::VARIABLE,
          tags: None,
          deprecated: None,
          range: variable.range,
          selection_range: variable.name.range,
          children: None,
        });
      }

      for function in document.functions() {
        let parameters = function
          .parameters
          .iter()
          .map(|parameter| parameter.value.as_str())
          .collect::<Vec<_>>()
          .join(", ");

        #[allow(deprecated)]
        symbols.push(lsp::DocumentSymbol {
          name: function.name.value,
          detail: Some(format!("({parameters})")),
          kind: lsp::SymbolKind::FUNCTION,
          tags: None,
          deprecated: None,
          range: function.range,
          selection_range: function.name.range,
          children: None,
        });
      }

      for setting in document.settings() {
        #[allow(deprecated)]
        symbols.push(lsp::DocumentSymbol {
          name: setting.name.value,
          detail: Some(setting.kind.to_string()),
          kind: lsp::SymbolKind::PROPERTY,
          tags: None,
          deprecated: None,
          range: setting.range,
          selection_range: setting.range,
          children: None,
        });
      }

      symbols.sort_by_key(|s| s.range.start);

      return Ok(Some(lsp::DocumentSymbolResponse::Nested(symbols)));
    }

    Ok(None)
  }

  async fn execute_command(
    &self,
    params: lsp::ExecuteCommandParams,
  ) -> Result<Option<serde_json::Value>, jsonrpc::Error> {
    self.executor.execute(params).await;

    Ok(None)
  }

  async fn folding_range(
    &self,
    params: lsp::FoldingRangeParams,
  ) -> Result<Option<Vec<lsp::FoldingRange>>, jsonrpc::Error> {
    let uri = &params.text_document.uri;

    let workspace = self.workspace.read().await;

    if let Some(document) = workspace.documents.get_open(uri) {
      let recipes = document.recipes();

      let folding_ranges = recipes
        .into_iter()
        .map(|recipe| {
          let start_line = recipe.range.start.line;

          let end_line = recipe.range.end.line;

          if end_line > start_line {
            lsp::FoldingRange {
              start_line,
              end_line: end_line.saturating_sub(1),
              kind: Some(lsp::FoldingRangeKind::Region),
              ..Default::default()
            }
          } else {
            lsp::FoldingRange {
              start_line,
              end_line: start_line,
              kind: Some(lsp::FoldingRangeKind::Region),
              ..Default::default()
            }
          }
        })
        .collect();

      return Ok(Some(folding_ranges));
    }

    Ok(None)
  }

  async fn formatting(
    &self,
    params: lsp::DocumentFormattingParams,
  ) -> Result<Option<Vec<lsp::TextEdit>>, jsonrpc::Error> {
    let config = self.config.read().await;

    let workspace = self.workspace.read().await;

    let Some(document) =
      workspace.documents.get_open(&params.text_document.uri)
    else {
      return Ok(None);
    };

    let content = document.content.to_string();

    match document.format(&config.formatting) {
      Ok(formatted) if formatted == content => Ok(Some(vec![])),
      Ok(formatted) => {
        let end = document
          .content
          .byte_to_lsp_position(document.content.len_bytes());

        Ok(Some(vec![lsp::TextEdit {
          range: lsp::Range {
            start: lsp::Position::new(0, 0),
            end,
          },
          new_text: formatted,
        }]))
      }
      Err(error) => {
        self
          .client
          .show_message(
            lsp::MessageType::ERROR,
            format!("Failed to format document: {error}"),
          )
          .await;

        Ok(None)
      }
    }
  }

  async fn goto_definition(
    &self,
    params: lsp::GotoDefinitionParams,
  ) -> Result<Option<lsp::GotoDefinitionResponse>, jsonrpc::Error> {
    let uri = params.text_document_position_params.text_document.uri;

    let position = params.text_document_position_params.position;

    let workspace = self.workspace.read().await;

    if let Some(target) = workspace
      .projects
      .get(&uri)
      .and_then(|project| project.dependency_at(&uri, position))
      .and_then(|dependency| match &dependency.target {
        ProjectDependencyTarget::Resolved(target) => Some(target),
        _ => None,
      })
    {
      return Ok(Some(lsp::GotoDefinitionResponse::Scalar(
        lsp::Location::new(target.clone(), lsp::Range::default()),
      )));
    }

    Ok(workspace.project_view(&uri).and_then(|view| {
      let identifier = view
        .document()
        .node_at_position(position)
        .filter(|node| node.kind() == "identifier")?;

      Resolver::new(view)
        .resolve_identifier_definition(&identifier)
        .map(lsp::GotoDefinitionResponse::Scalar)
    }))
  }

  async fn hover(
    &self,
    params: lsp::HoverParams,
  ) -> Result<Option<lsp::Hover>, jsonrpc::Error> {
    let uri = params.text_document_position_params.text_document.uri;

    let position = params.text_document_position_params.position;

    let workspace = self.workspace.read().await;

    Ok(workspace.project_view(&uri).and_then(|view| {
      let identifier = view
        .document()
        .node_at_position(position)
        .filter(|node| node.kind() == "identifier")?;

      Resolver::new(view).resolve_identifier_hover(&identifier)
    }))
  }

  async fn initialize(
    &self,
    params: lsp::InitializeParams,
  ) -> Result<lsp::InitializeResult, jsonrpc::Error> {
    info!("Starting just language server...");

    if let Some(options) = params.initialization_options {
      match serde_json::from_value::<Config>(options) {
        Ok(config) => *self.config.write().await = config,
        Err(error) => {
          warn!(%error, "failed to parse initialization options");
        }
      }
    }

    Ok(lsp::InitializeResult {
      capabilities: Server::capabilities(),
      server_info: Some(lsp::ServerInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
      }),
    })
  }

  async fn initialized(&self, _: lsp::InitializedParams) {
    self
      .client
      .log_message(
        lsp::MessageType::INFO,
        &format!("{} initialized", env!("CARGO_PKG_NAME")),
      )
      .await;

    self
      .initialized
      .store(true, std::sync::atomic::Ordering::Relaxed);
  }

  fn new(client: Client) -> Self {
    let executor = Executor::new(client.clone());

    Self {
      client,
      config: RwLock::new(Config::default()),
      executor,
      initialized: AtomicBool::new(false),
      workspace: RwLock::new(Workspace::default()),
    }
  }

  async fn prepare_rename(
    &self,
    params: lsp::TextDocumentPositionParams,
  ) -> Result<Option<lsp::PrepareRenameResponse>, jsonrpc::Error> {
    let uri = &params.text_document.uri;

    let workspace = self.workspace.read().await;

    Ok(workspace.documents.get_open(uri).and_then(|document| {
      document
        .node_at_position(params.position)
        .filter(|node| node.kind() == "identifier")
        .map(
          |identifier| lsp::PrepareRenameResponse::RangeWithPlaceholder {
            range: identifier.get_range(document),
            placeholder: document.get_node_text(&identifier),
          },
        )
    }))
  }

  async fn publish_diagnostics(&self, uri: &lsp::Url) {
    if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
      return;
    }

    let (diagnostics, version) = {
      let workspace = self.workspace.read().await;
      let config = self.config.read().await;

      match workspace.documents.get_open(uri) {
        Some(document) => {
          let imported_documents =
            workspace.projects.get(uri).into_iter().flat_map(|project| {
              project.imported_documents(&workspace.documents)
            });

          let analyzer = Analyzer {
            config: Some(&config),
            document,
            imported_documents: imported_documents.collect(),
          };

          (
            analyzer
              .analyze()
              .into_iter()
              .map(lsp::Diagnostic::from)
              .collect(),
            document.version,
          )
        }
        None => return,
      }
    };

    self
      .client
      .publish_diagnostics(uri.clone(), diagnostics, Some(version))
      .await;
  }

  async fn references(
    &self,
    params: lsp::ReferenceParams,
  ) -> Result<Option<Vec<lsp::Location>>, jsonrpc::Error> {
    let uri = params.text_document_position.text_document.uri;

    let position = params.text_document_position.position;

    let workspace = self.workspace.read().await;

    Ok(workspace.documents.get_open(&uri).and_then(|document| {
      let resolver = Resolver::new(document);

      document
        .node_at_position(position)
        .filter(|node| node.kind() == "identifier")
        .map(|identifier| resolver.resolve_identifier_references(&identifier))
    }))
  }

  async fn rename(
    &self,
    params: lsp::RenameParams,
  ) -> Result<Option<lsp::WorkspaceEdit>, jsonrpc::Error> {
    let uri = params.text_document_position.text_document.uri.clone();

    let position = params.text_document_position.position;

    let new_name = params.new_name;

    let workspace = self.workspace.read().await;

    Ok(workspace.documents.get_open(&uri).and_then(|document| {
      document
        .node_at_position(position)
        .filter(|node| node.kind() == "identifier")
        .map(|identifier| {
          let resolver = Resolver::new(document);

          let references = resolver.resolve_identifier_references(&identifier);

          let text_edits = references
            .iter()
            .map(|location| lsp::TextEdit {
              range: location.range,
              new_text: new_name.clone(),
            })
            .collect::<Vec<lsp::TextEdit>>();

          lsp::WorkspaceEdit {
            changes: Some(HashMap::from([(uri.clone(), text_edits)])),
            ..Default::default()
          }
        })
    }))
  }

  async fn semantic_tokens_full(
    &self,
    params: lsp::SemanticTokensParams,
  ) -> Result<Option<lsp::SemanticTokensResult>, jsonrpc::Error> {
    let uri = params.text_document.uri;

    let workspace = self.workspace.read().await;

    if let Some(document) = workspace.documents.get_open(&uri) {
      let tokenizer = Tokenizer::new(document);

      match tokenizer.tokenize() {
        Ok(data) => {
          return Ok(Some(lsp::SemanticTokensResult::Tokens(
            lsp::SemanticTokens {
              data,
              result_id: None,
            },
          )));
        }
        Err(error) => {
          self
            .client
            .log_message(
              lsp::MessageType::ERROR,
              format!("Failed to compute semantic tokens: {error}"),
            )
            .await;
        }
      }
    }

    Ok(None)
  }

  fn shutdown() -> impl future::Future<Output = Result<(), jsonrpc::Error>> {
    future::ready(Ok(()))
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::indoc,
    pretty_assertions::assert_eq,
    serde_json::{Value, json},
    std::env,
    tower_lsp::LspService,
    tower_test::mock::Spawn,
  };

  #[derive(Debug)]
  struct Test {
    requests: Vec<Value>,
    responses: Vec<Option<Value>>,
    service: Spawn<LspService<Server>>,
  }

  impl Test {
    fn new() -> Self {
      let (service, _) = LspService::new(Server::new);

      Self {
        requests: Vec::new(),
        responses: Vec::new(),
        service: Spawn::new(service),
      }
    }

    fn notification<T: IntoValue>(mut self, notification: T) -> Self {
      self.requests.push(notification.into_value());
      self.responses.push(None);
      self
    }

    fn request<T: IntoValue>(mut self, request: T) -> Self {
      self.requests.push(request.into_value());
      self
    }

    fn response<T: IntoValue>(mut self, response: T) -> Self {
      self.responses.push(Some(response.into_value()));
      self
    }

    async fn run(mut self) -> Result {
      for (request, expected_response) in
        self.requests.iter().zip(self.responses.iter())
      {
        let response = self
          .service
          .call(serde_json::from_value(request.clone())?)
          .await?;

        if let Some(expected) = expected_response {
          assert_eq!(
            *expected,
            response.map(|v| serde_json::to_value(v).unwrap()).unwrap()
          );
        } else {
          assert!(response.is_none(), "Expected no response for notification");
        }
      }

      Ok(())
    }
  }

  trait IntoValue {
    fn into_value(self) -> Value;
  }

  impl IntoValue for Value {
    fn into_value(self) -> Value {
      self
    }
  }

  #[derive(Debug)]
  struct InitializeRequest {
    id: i64,
  }

  impl IntoValue for InitializeRequest {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "method": "initialize",
        "params": {
          "capabilities": {}
        },
      })
    }
  }

  #[derive(Debug)]
  struct InitializeResponse {
    id: i64,
  }

  impl IntoValue for InitializeResponse {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "result": {
          "serverInfo": {
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION")
          },
          "capabilities": Server::capabilities()
        },
      })
    }
  }

  #[derive(Debug)]
  struct DidOpenNotification<'a> {
    text: &'a str,
    uri: &'a str,
  }

  impl IntoValue for DidOpenNotification<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
          "textDocument": {
            "uri": self.uri,
            "languageId": "just",
            "version": 1,
            "text": self.text
          }
        }
      })
    }
  }

  #[derive(Debug)]
  struct DidChangeNotification<'a> {
    changes: Vec<lsp::TextDocumentContentChangeEvent>,
    uri: &'a str,
    version: i32,
  }

  impl IntoValue for DidChangeNotification<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didChange",
        "params": {
          "textDocument": {
            "uri": self.uri,
            "version": self.version
          },
          "contentChanges": self.changes
        }
      })
    }
  }

  #[derive(Debug)]
  struct DidCloseNotification<'a> {
    uri: &'a str,
  }

  impl IntoValue for DidCloseNotification<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didClose",
        "params": {
          "textDocument": {
            "uri": self.uri
          }
        }
      })
    }
  }

  #[derive(Debug)]
  struct GotoDefinitionRequest<'a> {
    character: u32,
    id: i64,
    line: u32,
    uri: &'a str,
  }

  impl IntoValue for GotoDefinitionRequest<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "method": "textDocument/definition",
        "params": {
          "textDocument": {
            "uri": self.uri
          },
          "position": {
            "line": self.line,
            "character": self.character
          }
        }
      })
    }
  }

  #[derive(Debug)]
  struct GotoDefinitionResponse<'a> {
    end_char: u32,
    end_line: u32,
    id: i64,
    start_char: u32,
    start_line: u32,
    uri: &'a str,
  }

  impl IntoValue for GotoDefinitionResponse<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "result": {
          "uri": self.uri,
          "range": {
            "start": {
              "line": self.start_line,
              "character": self.start_char
            },
            "end": {
              "line": self.end_line,
              "character": self.end_char
            }
          }
        }
      })
    }
  }

  #[derive(Debug)]
  struct Location<'a> {
    end_char: u32,
    end_line: u32,
    start_char: u32,
    start_line: u32,
    uri: &'a str,
  }

  impl IntoValue for Location<'_> {
    fn into_value(self) -> Value {
      json!({
        "uri": self.uri,
        "range": {
          "start": {
            "line": self.start_line,
            "character": self.start_char
          },
          "end": {
            "line": self.end_line,
            "character": self.end_char
          }
        }
      })
    }
  }

  #[derive(Debug)]
  struct ReferencesRequest<'a> {
    character: u32,
    id: i64,
    include_declaration: bool,
    line: u32,
    uri: &'a str,
  }

  impl IntoValue for ReferencesRequest<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "method": "textDocument/references",
        "params": {
          "textDocument": {
            "uri": self.uri
          },
          "position": {
            "line": self.line,
            "character": self.character
          },
          "context": {
            "includeDeclaration": self.include_declaration
          }
        }
      })
    }
  }

  #[derive(Debug)]
  struct ReferencesResponse<'a> {
    id: i64,
    locations: Vec<Location<'a>>,
  }

  impl IntoValue for Vec<Location<'_>> {
    fn into_value(self) -> Value {
      self.into_iter().map(Location::into_value).collect()
    }
  }

  impl IntoValue for ReferencesResponse<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "result": self.locations.into_value()
      })
    }
  }

  #[derive(Debug)]
  struct Rename<'a> {
    end_char: u32,
    end_line: u32,
    new_text: &'a str,
    start_char: u32,
    start_line: u32,
  }

  impl IntoValue for Rename<'_> {
    fn into_value(self) -> Value {
      json!({
        "range": {
          "start": {
            "line": self.start_line,
            "character": self.start_char
          },
          "end": {
            "line": self.end_line,
            "character": self.end_char
          }
        },
        "newText": self.new_text
      })
    }
  }

  #[derive(Debug)]
  struct RenameRequest<'a> {
    character: u32,
    id: i64,
    line: u32,
    new_name: &'a str,
    uri: &'a str,
  }

  impl IntoValue for RenameRequest<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "method": "textDocument/rename",
        "params": {
          "textDocument": {
            "uri": self.uri
          },
          "position": {
            "line": self.line,
            "character": self.character
          },
          "newName": self.new_name
        }
      })
    }
  }

  #[derive(Debug)]
  struct RenameResponse<'a> {
    edits: Vec<Rename<'a>>,
    id: i64,
    uri: &'a str,
  }

  impl IntoValue for Vec<Rename<'_>> {
    fn into_value(self) -> Value {
      self.into_iter().map(Rename::into_value).collect()
    }
  }

  impl IntoValue for RenameResponse<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "result": {
          "changes": {
            self.uri: self.edits.into_value()
          }
        }
      })
    }
  }

  #[derive(Debug)]
  struct PrepareRenameRequest<'a> {
    character: u32,
    id: i64,
    line: u32,
    uri: &'a str,
  }

  impl IntoValue for PrepareRenameRequest<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "method": "textDocument/prepareRename",
        "params": {
          "textDocument": {
            "uri": self.uri
          },
          "position": {
            "line": self.line,
            "character": self.character
          }
        }
      })
    }
  }

  #[derive(Debug)]
  struct PrepareRenameResponse<'a> {
    end_char: u32,
    end_line: u32,
    id: i64,
    placeholder: &'a str,
    start_char: u32,
    start_line: u32,
  }

  impl IntoValue for PrepareRenameResponse<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "result": {
          "range": {
            "start": {
              "line": self.start_line,
              "character": self.start_char
            },
            "end": {
              "line": self.end_line,
              "character": self.end_char
            }
          },
          "placeholder": self.placeholder
        }
      })
    }
  }

  #[derive(Debug)]
  struct HoverRequest<'a> {
    character: u32,
    id: i64,
    line: u32,
    uri: &'a str,
  }

  impl IntoValue for HoverRequest<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "method": "textDocument/hover",
        "params": {
          "textDocument": {
            "uri": self.uri
          },
          "position": {
            "line": self.line,
            "character": self.character
          }
        }
      })
    }
  }

  #[derive(Debug)]
  struct HoverResponse<'a> {
    content: &'a str,
    end_char: u32,
    end_line: u32,
    id: i64,
    kind: &'a str,
    start_char: u32,
    start_line: u32,
  }

  impl IntoValue for HoverResponse<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "result": {
          "contents": {
            "kind": self.kind,
            "value": self.content
          },
          "range": {
            "start": {
              "line": self.start_line,
              "character": self.start_char
            },
            "end": {
              "line": self.end_line,
              "character": self.end_char
            }
          }
        }
      })
    }
  }

  #[derive(Debug)]
  struct DocumentHighlightRequest<'a> {
    character: u32,
    id: i64,
    line: u32,
    uri: &'a str,
  }

  impl IntoValue for DocumentHighlightRequest<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "method": "textDocument/documentHighlight",
        "params": {
          "textDocument": {
            "uri": self.uri
          },
          "position": {
            "line": self.line,
            "character": self.character
          }
        }
      })
    }
  }

  #[derive(Debug)]
  struct DocumentHighlightResponse<'a> {
    highlights: Vec<Highlight<'a>>,
    id: i64,
  }

  impl IntoValue for DocumentHighlightResponse<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "result": self.highlights.into_value()
      })
    }
  }

  #[derive(Debug)]
  struct Highlight<'a> {
    end_char: u32,
    end_line: u32,
    kind: &'a str,
    start_char: u32,
    start_line: u32,
  }

  impl IntoValue for Highlight<'_> {
    fn into_value(self) -> Value {
      json!({
        "range": {
          "start": {
            "line": self.start_line,
            "character": self.start_char
          },
          "end": {
            "line": self.end_line,
            "character": self.end_char
          }
        },
        "kind": match self.kind {
          "read" => 2,
          "write" => 3,
          _ => 1
        }
      })
    }
  }

  impl IntoValue for Vec<Highlight<'_>> {
    fn into_value(self) -> Value {
      self.into_iter().map(Highlight::into_value).collect()
    }
  }

  #[derive(Debug)]
  struct SemanticTokensRequest<'a> {
    id: i64,
    uri: &'a str,
  }

  impl IntoValue for SemanticTokensRequest<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "method": "textDocument/semanticTokens/full",
        "params": {
          "textDocument": {
            "uri": self.uri
          }
        }
      })
    }
  }

  #[derive(Debug)]
  struct SemanticTokensResponse {
    data: Vec<u32>,
    id: i64,
  }

  impl IntoValue for SemanticTokensResponse {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "result": {
          "data": self.data,
        }
      })
    }
  }

  #[derive(Debug)]
  struct FoldingRange<'a> {
    end_line: u32,
    kind: &'a str,
    start_line: u32,
  }

  impl IntoValue for FoldingRange<'_> {
    fn into_value(self) -> Value {
      json!({
        "startLine": self.start_line,
        "endLine": self.end_line,
        "kind": self.kind
      })
    }
  }

  impl IntoValue for Vec<FoldingRange<'_>> {
    fn into_value(self) -> Value {
      self.into_iter().map(FoldingRange::into_value).collect()
    }
  }

  #[derive(Debug)]
  struct FoldingRangeRequest<'a> {
    id: i64,
    uri: &'a str,
  }

  impl IntoValue for FoldingRangeRequest<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "method": "textDocument/foldingRange",
        "params": {
          "textDocument": {
            "uri": self.uri
          }
        }
      })
    }
  }

  #[derive(Debug)]
  struct FoldingRangeResponse<'a> {
    id: i64,
    ranges: Vec<FoldingRange<'a>>,
  }

  impl IntoValue for FoldingRangeResponse<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "result": self.ranges.into_value()
      })
    }
  }

  #[derive(Debug)]
  struct CodeActionRequest {
    id: i64,
    range: lsp::Range,
    uri: &'static str,
  }

  impl IntoValue for CodeActionRequest {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "method": "textDocument/codeAction",
        "params": {
          "textDocument": {
            "uri": self.uri
          },
          "range": {
            "start": {
              "line": self.range.start.line,
              "character": self.range.start.character
            },
            "end": {
              "line": self.range.end.line,
              "character": self.range.end.character
            }
          },
          "context": {
            "diagnostics": []
          }
        }
      })
    }
  }

  #[derive(Debug)]
  struct CodeActionResponse {
    actions: Vec<CodeAction>,
    id: i64,
  }

  impl IntoValue for CodeActionResponse {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "result": self.actions.into_value()
      })
    }
  }

  #[derive(Debug)]
  struct CodeAction {
    arguments: Vec<ParameterJson>,
    command: Command,
    kind: &'static str,
    title: &'static str,
  }

  impl IntoValue for Vec<ParameterJson> {
    fn into_value(self) -> Value {
      self
        .into_iter()
        .map(|p| serde_json::to_value(p).unwrap())
        .collect()
    }
  }

  impl IntoValue for CodeAction {
    fn into_value(self) -> Value {
      let recipe_name = json!(self.title);

      let uri = json!("file:///test.just");

      let parameters = json!(self.arguments.into_value());

      json!({
        "title": self.title,
        "kind": self.kind,
        "command": {
          "title": self.title,
          "command": self.command.to_string(),
          "arguments": [recipe_name, uri, parameters]
        }
      })
    }
  }

  impl IntoValue for Vec<CodeAction> {
    fn into_value(self) -> Value {
      self.into_iter().map(CodeAction::into_value).collect()
    }
  }

  #[derive(Debug)]
  struct CodeLensRequest {
    id: i64,
    uri: &'static str,
  }

  impl IntoValue for CodeLensRequest {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "method": "textDocument/codeLens",
        "params": {
          "textDocument": {
            "uri": self.uri
          }
        }
      })
    }
  }

  #[tokio::test]
  async fn closing_imported_buffer_restores_disk_project() -> Result {
    let tempdir = tempfile::tempdir()?;

    let root =
      lsp::Url::from_file_path(tempdir.path().join("justfile")).unwrap();

    let imported =
      lsp::Url::from_file_path(tempdir.path().join("foo.just")).unwrap();

    let target =
      lsp::Url::from_file_path(tempdir.path().join("bar.just")).unwrap();

    std::fs::write(imported.to_file_path().unwrap(), "import 'bar.just'")?;
    std::fs::write(target.to_file_path().unwrap(), "bar:")?;

    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: root.as_str(),
        text: "import 'foo.just'\n\nfoo: bar",
      })
      .notification(DidOpenNotification {
        uri: imported.as_str(),
        text: "",
      })
      .request(HoverRequest {
        id: 2,
        uri: root.as_str(),
        line: 2,
        character: 5,
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": null
      }))
      .notification(DidCloseNotification {
        uri: imported.as_str(),
      })
      .request(HoverRequest {
        id: 3,
        uri: root.as_str(),
        line: 2,
        character: 5,
      })
      .response(HoverResponse {
        id: 3,
        content: "bar:",
        kind: "plaintext",
        start_line: 2,
        start_char: 5,
        end_line: 2,
        end_char: 8,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn code_action_deprecated_function_or_default_quickfix() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: "foo := env_var_or_default(\"BAR\", \"baz\")\n",
      })
      .request(CodeActionRequest {
        id: 2,
        uri: "file:///test.just",
        range: lsp::Range::at(0, 10, 0, 10),
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": [
          {
            "title": "Replace `env_var_or_default` with `env`",
            "kind": "quickfix",
            "edit": {
              "changes": {
                "file:///test.just": [
                  {
                    "range": {
                      "start": { "line": 0, "character": 7 },
                      "end": { "line": 0, "character": 25 }
                    },
                    "newText": "env"
                  }
                ]
              }
            }
          }
        ]
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn code_action_deprecated_function_outside_range() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: "foo := env(\"BAR\")\n",
      })
      .notification(DidChangeNotification {
        uri: "file:///test.just",
        version: 2,
        changes: vec![lsp::TextDocumentContentChangeEvent {
          range: Some(lsp::Range::at(0, 7, 0, 10)),
          range_length: None,
          text: "env_var".into(),
        }],
      })
      .request(CodeActionRequest {
        id: 2,
        uri: "file:///test.just",
        range: lsp::Range::at(0, 0, 0, 3),
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": []
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn code_action_deprecated_function_quickfix() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: "foo := env_var(\"BAR\")\n",
      })
      .request(CodeActionRequest {
        id: 2,
        uri: "file:///test.just",
        range: lsp::Range::at(0, 10, 0, 10),
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": [
          {
            "title": "Replace `env_var` with `env`",
            "kind": "quickfix",
            "edit": {
              "changes": {
                "file:///test.just": [
                  {
                    "range": {
                      "start": { "line": 0, "character": 7 },
                      "end": { "line": 0, "character": 14 }
                    },
                    "newText": "env"
                  }
                ]
              }
            }
          }
        ]
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn code_action_empty_document() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///empty.just",
        text: "",
      })
      .request(CodeActionRequest {
        id: 2,
        uri: "file:///empty.just",
        range: lsp::Range::at(0, 0, 0, 0),
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": []
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn code_action_with_recipes() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo foo

          bar arg1 arg2='default':
            echo bar
          "
        },
      })
      .request(CodeActionRequest {
        id: 2,
        uri: "file:///test.just",
        range: lsp::Range::at(0, 0, 0, 0),
      })
      .response(CodeActionResponse {
        id: 2,
        actions: vec![
          CodeAction {
            title: "foo",
            kind: "source",
            command: Command::RunRecipe,
            arguments: vec![],
          },
          CodeAction {
            title: "bar",
            kind: "source",
            command: Command::RunRecipe,
            arguments: vec![
              ParameterJson {
                name: "arg1".into(),
                default_value: None,
              },
              ParameterJson {
                name: "arg2".into(),
                default_value: Some("'default'".to_string()),
              },
            ],
          },
        ],
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn code_lens_empty_document() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///empty.just",
        text: "",
      })
      .request(CodeLensRequest {
        id: 2,
        uri: "file:///empty.just",
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": []
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn code_lens_with_recipes() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo foo

          bar arg1 arg2='default':
            echo bar
          "
        },
      })
      .request(CodeLensRequest {
        id: 2,
        uri: "file:///test.just",
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": [
          {
            "range": {
              "start": { "line": 0, "character": 0 },
              "end": { "line": 0, "character": 3 }
            },
            "command": {
              "title": "Run",
              "command": "just-lsp.run_recipe",
              "arguments": ["foo", "file:///test.just", []]
            }
          },
          {
            "range": {
              "start": { "line": 3, "character": 0 },
              "end": { "line": 3, "character": 3 }
            },
            "command": {
              "title": "Run",
              "command": "just-lsp.run_recipe",
              "arguments": [
                "bar",
                "file:///test.just",
                [
                  { "name": "arg1", "default_value": null },
                  { "name": "arg2", "default_value": "'default'" }
                ]
              ]
            }
          }
        ]
      }))
      .run()
      .await
  }

  #[derive(Debug)]
  struct DocumentLinkRequest<'a> {
    id: i64,
    uri: &'a str,
  }

  impl IntoValue for DocumentLinkRequest<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "method": "textDocument/documentLink",
        "params": {
          "textDocument": {
            "uri": self.uri
          }
        }
      })
    }
  }

  #[tokio::test]
  async fn dependency_open_republishes_root_diagnostics() -> Result {
    let tempdir = tempfile::tempdir()?;

    let root =
      lsp::Url::from_file_path(tempdir.path().join("justfile")).unwrap();

    let imported =
      lsp::Url::from_file_path(tempdir.path().join("foo.just")).unwrap();

    std::fs::write(imported.to_file_path().unwrap(), "")?;

    let (service, mut socket) = LspService::new(Server::new);

    let mut service = Spawn::new(service);

    service
      .call(serde_json::from_value(
        InitializeRequest { id: 1 }.into_value(),
      )?)
      .await?;

    let initialized = service.call(serde_json::from_value(json!({
      "jsonrpc": "2.0",
      "method": "initialized",
      "params": {}
    }))?);

    let (response, _) = tokio::join!(initialized, socket.next());

    response?;

    let open = service.call(serde_json::from_value(
      DidOpenNotification {
        uri: root.as_str(),
        text: "import 'foo.just'\n\nbar: foo",
      }
      .into_value(),
    )?);

    let (response, diagnostics) = tokio::join!(open, socket.next());

    response?;

    let diagnostics = serde_json::to_value(diagnostics.unwrap())?;

    assert_eq!(diagnostics["method"], "textDocument/publishDiagnostics");
    assert_eq!(diagnostics["params"]["uri"], root.as_str());

    assert!(
      !diagnostics["params"]["diagnostics"]
        .as_array()
        .unwrap()
        .is_empty()
    );

    let open = service.call(serde_json::from_value(
      DidOpenNotification {
        uri: imported.as_str(),
        text: "foo:",
      }
      .into_value(),
    )?);

    let diagnostics =
      async { [socket.next().await.unwrap(), socket.next().await.unwrap()] };

    let (response, diagnostics) = tokio::join!(open, diagnostics);

    response?;

    let diagnostics = diagnostics
      .into_iter()
      .map(serde_json::to_value)
      .collect::<serde_json::Result<Vec<_>>>()?;

    let diagnostics = diagnostics
      .iter()
      .find(|diagnostics| diagnostics["params"]["uri"] == root.as_str())
      .unwrap();

    assert_eq!(diagnostics["params"]["diagnostics"], json!([]));

    Ok(())
  }

  #[tokio::test]
  async fn did_change_updates_document() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo \"foo\"
          "
        },
      })
      .notification(DidChangeNotification {
        uri: "file:///test.just",
        version: 2,
        changes: vec![lsp::TextDocumentContentChangeEvent {
          range: Some(lsp::Range::at(1, 7, 1, 13)),
          range_length: None,
          text: "\"updated\"".into(),
        }],
      })
      .request(HoverRequest {
        id: 2,
        uri: "file:///test.just",
        line: 0,
        character: 1,
      })
      .response(HoverResponse {
        id: 2,
        content: "foo:\n  echo \"updated\"",
        kind: "plaintext",
        start_line: 0,
        start_char: 0,
        end_line: 0,
        end_char: 3,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn did_change_without_open_document_is_ignored() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidChangeNotification {
        uri: "file:///missing.just",
        version: 2,
        changes: vec![lsp::TextDocumentContentChangeEvent {
          range: Some(lsp::Range::at(0, 0, 0, 0)),
          range_length: None,
          text: "\"updated\"".into(),
        }],
      })
      .notification(DidOpenNotification {
        uri: "file:///missing.just",
        text: indoc! {
          "
          foo:
            echo \"foo\"

          bar: foo
            echo \"bar\"
          "
        },
      })
      .request(HoverRequest {
        id: 2,
        uri: "file:///missing.just",
        line: 3,
        character: 5,
      })
      .response(HoverResponse {
        id: 2,
        content: "foo:\n  echo \"foo\"",
        kind: "plaintext",
        start_line: 3,
        start_char: 5,
        end_line: 3,
        end_char: 8,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn document_highlight() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo \"foo\"

          bar: foo
            echo \"bar\"

          alias baz := foo
          "
        },
      })
      .request(DocumentHighlightRequest {
        id: 2,
        uri: "file:///test.just",
        line: 0,
        character: 1,
      })
      .response(DocumentHighlightResponse {
        id: 2,
        highlights: vec![
          Highlight {
            start_line: 0,
            start_char: 0,
            end_line: 0,
            end_char: 3,
            kind: "text",
          },
          Highlight {
            start_line: 3,
            start_char: 5,
            end_line: 3,
            end_char: 8,
            kind: "text",
          },
          Highlight {
            start_line: 6,
            start_char: 13,
            end_line: 6,
            end_char: 16,
            kind: "text",
          },
        ],
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn document_link_empty_document() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: "",
      })
      .request(DocumentLinkRequest {
        id: 2,
        uri: "file:///test.just",
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": []
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn document_link_ignores_unresolved_imports() -> Result {
    let tempdir = tempfile::tempdir()?;

    let root =
      lsp::Url::from_file_path(tempdir.path().join("justfile")).unwrap();

    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: root.as_str(),
        text: "import? 'missing.just'\nimport x'dynamic.just'\n",
      })
      .request(DocumentLinkRequest {
        id: 2,
        uri: root.as_str(),
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": []
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn document_link_import() -> Result {
    let tempdir = tempfile::tempdir()?;

    let root = tempdir.path().join("justfile");
    let target = tempdir.path().join("bar.just");

    std::fs::write(&target, "bar:")?;

    let root_uri = lsp::Url::from_file_path(root).unwrap();

    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: root_uri.as_str(),
        text: "import 'bar.just'\n",
      })
      .request(DocumentLinkRequest {
        id: 2,
        uri: root_uri.as_str(),
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": [
          {
            "range": {
              "start": { "line": 0, "character": 7 },
              "end": { "line": 0, "character": 17 }
            },
            "target": lsp::Url::from_file_path(&target).unwrap(),
            "tooltip": target.display().to_string()
          }
        ]
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn document_link_module_with_path() -> Result {
    let (justfile_uri, target_uri, tooltip) = if cfg!(windows) {
      (
        "file:///C:/foo/justfile",
        "file:///C:/foo/baz.just",
        "C:\\foo\\baz.just",
      )
    } else {
      (
        "file:///foo/justfile",
        "file:///foo/baz.just",
        "/foo/baz.just",
      )
    };

    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: justfile_uri,
        text: "mod bar 'baz.just'\n",
      })
      .request(DocumentLinkRequest {
        id: 2,
        uri: justfile_uri,
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": [
          {
            "range": {
              "start": { "line": 0, "character": 8 },
              "end": { "line": 0, "character": 18 }
            },
            "target": target_uri,
            "tooltip": tooltip
          }
        ]
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn document_symbol_empty_document() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///empty.just",
        text: "",
      })
      .request(DocumentSymbolRequest {
        id: 2,
        uri: "file:///empty.just",
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": []
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn document_symbol_with_alias() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo foo

          alias bar := foo
          "
        },
      })
      .request(DocumentSymbolRequest {
        id: 2,
        uri: "file:///test.just",
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": [
          {
            "name": "foo",
            "kind": 12,
            "range": {
              "start": { "line": 0, "character": 0 },
              "end": { "line": 3, "character": 0 }
            },
            "selectionRange": {
              "start": { "line": 0, "character": 0 },
              "end": { "line": 0, "character": 3 }
            }
          },
          {
            "name": "bar",
            "detail": "alias for foo",
            "kind": 12,
            "range": {
              "start": { "line": 3, "character": 0 },
              "end": { "line": 3, "character": 16 }
            },
            "selectionRange": {
              "start": { "line": 3, "character": 6 },
              "end": { "line": 3, "character": 9 }
            }
          }
        ]
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn document_symbol_with_recipes_and_variables() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          bar := 'baz'

          foo:
            echo foo
          "
        },
      })
      .request(DocumentSymbolRequest {
        id: 2,
        uri: "file:///test.just",
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": [
          {
            "name": "bar",
            "kind": 13,
            "range": {
              "start": { "line": 0, "character": 0 },
              "end": { "line": 1, "character": 0 }
            },
            "selectionRange": {
              "start": { "line": 0, "character": 0 },
              "end": { "line": 0, "character": 3 }
            }
          },
          {
            "name": "foo",
            "kind": 12,
            "range": {
              "start": { "line": 2, "character": 0 },
              "end": { "line": 4, "character": 0 }
            },
            "selectionRange": {
              "start": { "line": 2, "character": 0 },
              "end": { "line": 2, "character": 3 }
            }
          }
        ]
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn document_symbol_with_setting() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          set export := true

          foo:
            echo foo
          "
        },
      })
      .request(DocumentSymbolRequest {
        id: 2,
        uri: "file:///test.just",
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": [
          {
            "name": "export",
            "detail": "boolean",
            "kind": 7,
            "range": {
              "start": { "line": 0, "character": 0 },
              "end": { "line": 1, "character": 0 }
            },
            "selectionRange": {
              "start": { "line": 0, "character": 0 },
              "end": { "line": 1, "character": 0 }
            }
          },
          {
            "name": "foo",
            "kind": 12,
            "range": {
              "start": { "line": 2, "character": 0 },
              "end": { "line": 4, "character": 0 }
            },
            "selectionRange": {
              "start": { "line": 2, "character": 0 },
              "end": { "line": 2, "character": 3 }
            }
          }
        ]
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn folding_range() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo \"foo\"
            echo \"another line\"

          bar:
            echo \"bar\"
          "
        },
      })
      .request(FoldingRangeRequest {
        id: 2,
        uri: "file:///test.just",
      })
      .response(FoldingRangeResponse {
        id: 2,
        ranges: vec![
          FoldingRange {
            start_line: 0,
            end_line: 3,
            kind: "region",
          },
          FoldingRange {
            start_line: 4,
            end_line: 5,
            kind: "region",
          },
        ],
      })
      .run()
      .await
  }

  #[derive(Debug)]
  struct DocumentSymbolRequest<'a> {
    id: i64,
    uri: &'a str,
  }

  impl IntoValue for DocumentSymbolRequest<'_> {
    fn into_value(self) -> Value {
      json!({
        "jsonrpc": "2.0",
        "id": self.id,
        "method": "textDocument/documentSymbol",
        "params": {
          "textDocument": {
            "uri": self.uri
          }
        }
      })
    }
  }

  #[tokio::test]
  async fn goto_import_definition() -> Result {
    let tempdir = tempfile::tempdir()?;

    let root = tempdir.path().join("justfile");
    let target = tempdir.path().join("foo.just");

    std::fs::write(&target, "foo:")?;

    let root = lsp::Url::from_file_path(root).unwrap();

    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: root.as_str(),
        text: "import 'foo.just'",
      })
      .request(GotoDefinitionRequest {
        id: 2,
        uri: root.as_str(),
        line: 0,
        character: 9,
      })
      .response(GotoDefinitionResponse {
        id: 2,
        uri: lsp::Url::from_file_path(target).unwrap().as_str(),
        start_line: 0,
        start_char: 0,
        end_line: 0,
        end_char: 0,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn goto_recipe_definition_from_dependency() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo \"foo\"

          bar: foo
            echo \"bar\"
          "
        },
      })
      .request(GotoDefinitionRequest {
        id: 2,
        uri: "file:///test.just",
        line: 3,
        character: 5,
      })
      .response(GotoDefinitionResponse {
        id: 2,
        uri: "file:///test.just",
        start_line: 0,
        start_char: 0,
        end_line: 3,
        end_char: 0,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_attribute() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          [no-cd]
          foo:
            echo \"foo\"
          "
        },
      })
      .request(HoverRequest {
        id: 2,
        uri: "file:///test.just",
        line: 0,
        character: 3,
      })
      .response(HoverResponse {
        id: 2,
        content: indoc! {
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
        },
        kind: "markdown",
        start_line: 0,
        start_char: 1,
        end_line: 0,
        end_char: 6,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_builtin_function() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo {{arch()}}
          "
        },
      })
      .request(HoverRequest {
        id: 2,
        uri: "file:///test.just",
        line: 1,
        character: 11,
      })
      .response(HoverResponse {
        id: 2,
        content: indoc! {
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
        },
        kind: "markdown",
        start_line: 1,
        start_char: 9,
        end_line: 1,
        end_char: 13,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_constant() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo {{ HEX }}

          bar: foo
            echo \"bar\"
          "
        },
      })
      .request(HoverRequest {
        id: 2,
        uri: "file:///test.just",
        line: 1,
        character: 12,
      })
      .response(HoverResponse {
        id: 2,
        content: indoc! {
          "
          Lowercase hexadecimal digit string: `\"0123456789abcdef\"`.

          Useful as the alphabet argument to `choose()` for generating
          random hex strings.

          ```just
          token := choose('32', HEX)
          ```
          "
        },
        kind: "markdown",
        start_line: 1,
        start_char: 10,
        end_line: 1,
        end_char: 13,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_local_parameter() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          bar arg='cooler':
            echo {{ arg }}

          foo arg='cool':
            echo {{ arg }}
          "
        },
      })
      .request(HoverRequest {
        id: 2,
        uri: "file:///test.just",
        line: 4,
        character: 11,
      })
      .response(HoverResponse {
        id: 2,
        content: "arg='cool'",
        kind: "plaintext",
        start_line: 4,
        start_char: 10,
        end_line: 4,
        end_char: 13,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_prioritize_recipe_parameter_over_variable_in_interpolation()
  -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          arg := 'wow'

          foo arg='cool':
            echo {{ arg }}
          "
        },
      })
      .request(HoverRequest {
        id: 2,
        uri: "file:///test.just",
        line: 3,
        character: 11,
      })
      .response(HoverResponse {
        id: 2,
        content: "arg='cool'",
        kind: "plaintext",
        start_line: 3,
        start_char: 10,
        end_line: 3,
        end_char: 13,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_recipe() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo \"foo\"

          bar: foo
            echo \"bar\"
          "
        },
      })
      .request(HoverRequest {
        id: 2,
        uri: "file:///test.just",
        line: 3,
        character: 5,
      })
      .response(HoverResponse {
        id: 2,
        content: "foo:\n  echo \"foo\"",
        kind: "plaintext",
        start_line: 3,
        start_char: 5,
        end_line: 3,
        end_char: 8,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_recipe_parameter_in_interpolation() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo arg='cool':
            echo {{ arg }}
          "
        },
      })
      .request(HoverRequest {
        id: 2,
        uri: "file:///test.just",
        line: 1,
        character: 11,
      })
      .response(HoverResponse {
        id: 2,
        content: "arg='cool'",
        kind: "plaintext",
        start_line: 1,
        start_char: 10,
        end_line: 1,
        end_char: 13,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_same_named_recipes_and_functions() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          arch:
            echo \"foo\"

          bar: arch
            echo {{ arch() }}
          "
        },
      })
      .request(HoverRequest {
        id: 2,
        uri: "file:///test.just",
        line: 3,
        character: 5,
      })
      .response(HoverResponse {
        id: 2,
        content: "arch:\n  echo \"foo\"",
        kind: "plaintext",
        start_line: 3,
        start_char: 5,
        end_line: 3,
        end_char: 9,
      })
      .request(HoverRequest {
        id: 2,
        uri: "file:///test.just",
        line: 4,
        character: 11,
      })
      .response(HoverResponse {
        id: 2,
        content: indoc! {
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
        },
        kind: "markdown",
        start_line: 4,
        start_char: 10,
        end_line: 4,
        end_char: 14,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_setting() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          set export := true

          foo:
            echo \"foo\"
          "
        },
      })
      .request(HoverRequest {
        id: 2,
        uri: "file:///test.just",
        line: 0,
        character: 4,
      })
      .response(HoverResponse {
        id: 2,
        content: indoc! {
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
        },
        kind: "markdown",
        start_line: 0,
        start_char: 4,
        end_line: 0,
        end_char: 10,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_variable_in_interpolation() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo := 'foo'

          foo:
            echo {{ foo }}
          "
        },
      })
      .request(HoverRequest {
        id: 2,
        uri: "file:///test.just",
        line: 3,
        character: 11,
      })
      .response(HoverResponse {
        id: 2,
        content: "foo := 'foo'",
        kind: "plaintext",
        start_line: 3,
        start_char: 10,
        end_line: 3,
        end_char: 13,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn imported_symbol_navigation() -> Result {
    let tempdir = tempfile::tempdir()?;

    let root = tempdir.path().join("justfile");
    let target = tempdir.path().join("foo.just");

    std::fs::write(
      &target,
      indoc! {
        "
        qux:
          echo foo

        bar := 'baz'

        qux() := 'quux'
        "
      },
    )?;

    let root = lsp::Url::from_file_path(root).unwrap();
    let target = lsp::Url::from_file_path(target).unwrap();

    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: root.as_str(),
        text: indoc! {
          "
          import 'foo.just'

          foo bar='local': qux
            echo {{ bar }}
            echo {{ qux() }}

          baz:
            echo {{ bar }}
          "
        },
      })
      .request(GotoDefinitionRequest {
        id: 2,
        uri: root.as_str(),
        line: 2,
        character: 18,
      })
      .response(GotoDefinitionResponse {
        id: 2,
        uri: target.as_str(),
        start_line: 0,
        start_char: 0,
        end_line: 3,
        end_char: 0,
      })
      .request(GotoDefinitionRequest {
        id: 3,
        uri: root.as_str(),
        line: 4,
        character: 11,
      })
      .response(GotoDefinitionResponse {
        id: 3,
        uri: target.as_str(),
        start_line: 5,
        start_char: 0,
        end_line: 5,
        end_char: 3,
      })
      .request(GotoDefinitionRequest {
        id: 4,
        uri: root.as_str(),
        line: 7,
        character: 11,
      })
      .response(GotoDefinitionResponse {
        id: 4,
        uri: target.as_str(),
        start_line: 3,
        start_char: 0,
        end_line: 4,
        end_char: 0,
      })
      .request(HoverRequest {
        id: 5,
        uri: root.as_str(),
        line: 2,
        character: 18,
      })
      .response(HoverResponse {
        id: 5,
        content: "qux:\n  echo foo",
        kind: "plaintext",
        start_line: 2,
        start_char: 17,
        end_line: 2,
        end_char: 20,
      })
      .request(HoverRequest {
        id: 6,
        uri: root.as_str(),
        line: 4,
        character: 11,
      })
      .response(HoverResponse {
        id: 6,
        content: "qux() := 'quux'",
        kind: "plaintext",
        start_line: 4,
        start_char: 10,
        end_line: 4,
        end_char: 13,
      })
      .request(HoverRequest {
        id: 7,
        uri: root.as_str(),
        line: 7,
        character: 11,
      })
      .response(HoverResponse {
        id: 7,
        content: "bar := 'baz'",
        kind: "plaintext",
        start_line: 7,
        start_char: 10,
        end_line: 7,
        end_char: 13,
      })
      .request(HoverRequest {
        id: 8,
        uri: root.as_str(),
        line: 3,
        character: 11,
      })
      .response(HoverResponse {
        id: 8,
        content: "bar='local'",
        kind: "plaintext",
        start_line: 3,
        start_char: 10,
        end_line: 3,
        end_char: 13,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn imported_symbol_navigation_rebuilds_affected_roots() -> Result {
    let tempdir = tempfile::tempdir()?;

    let first =
      lsp::Url::from_file_path(tempdir.path().join("foo.just")).unwrap();

    let second =
      lsp::Url::from_file_path(tempdir.path().join("bar.just")).unwrap();

    let imported =
      lsp::Url::from_file_path(tempdir.path().join("baz.just")).unwrap();

    let target =
      lsp::Url::from_file_path(tempdir.path().join("qux.just")).unwrap();

    std::fs::write(target.to_file_path().unwrap(), "qux:")?;

    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: first.as_str(),
        text: "import 'baz.just'\n\nfoo: qux",
      })
      .notification(DidOpenNotification {
        uri: second.as_str(),
        text: "import 'baz.just'\n\nbar: qux",
      })
      .notification(DidOpenNotification {
        uri: imported.as_str(),
        text: "",
      })
      .request(HoverRequest {
        id: 2,
        uri: first.as_str(),
        line: 2,
        character: 5,
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": null
      }))
      .notification(DidChangeNotification {
        uri: imported.as_str(),
        version: 2,
        changes: vec![lsp::TextDocumentContentChangeEvent {
          range: None,
          range_length: None,
          text: "import 'qux.just'".into(),
        }],
      })
      .request(HoverRequest {
        id: 3,
        uri: first.as_str(),
        line: 2,
        character: 5,
      })
      .response(HoverResponse {
        id: 3,
        content: "qux:",
        kind: "plaintext",
        start_line: 2,
        start_char: 5,
        end_line: 2,
        end_char: 8,
      })
      .request(HoverRequest {
        id: 4,
        uri: second.as_str(),
        line: 2,
        character: 5,
      })
      .response(HoverResponse {
        id: 4,
        content: "qux:",
        kind: "plaintext",
        start_line: 2,
        start_char: 5,
        end_line: 2,
        end_char: 8,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn imported_symbol_navigation_uses_open_buffer() -> Result {
    let tempdir = tempfile::tempdir()?;

    let root = tempdir.path().join("justfile");
    let target = tempdir.path().join("foo.just");

    std::fs::write(&target, "foo:\n  echo disk")?;

    let root = lsp::Url::from_file_path(root).unwrap();
    let target = lsp::Url::from_file_path(target).unwrap();

    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: root.as_str(),
        text: "import 'foo.just'\n\nbar: foo",
      })
      .notification(DidOpenNotification {
        uri: target.as_str(),
        text: "\nfoo:\n  echo buffer",
      })
      .request(GotoDefinitionRequest {
        id: 2,
        uri: root.as_str(),
        line: 2,
        character: 5,
      })
      .response(GotoDefinitionResponse {
        id: 2,
        uri: target.as_str(),
        start_line: 1,
        start_char: 0,
        end_line: 2,
        end_char: 13,
      })
      .request(HoverRequest {
        id: 3,
        uri: root.as_str(),
        line: 2,
        character: 5,
      })
      .response(HoverResponse {
        id: 3,
        content: "foo:\n  echo buffer",
        kind: "plaintext",
        start_line: 2,
        start_char: 5,
        end_line: 2,
        end_char: 8,
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn initialize() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .run()
      .await
  }

  #[tokio::test]
  async fn initialize_once() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .request(InitializeRequest { id: 1 })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
          "code": -32600,
          "message": "Invalid request"
        }
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn prepare_rename_identifier() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo \"foo\"
          "
        },
      })
      .request(PrepareRenameRequest {
        id: 2,
        uri: "file:///test.just",
        line: 0,
        character: 1,
      })
      .response(PrepareRenameResponse {
        id: 2,
        start_line: 0,
        start_char: 0,
        end_line: 0,
        end_char: 3,
        placeholder: "foo",
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn prepare_rename_non_identifier() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo \"foo\"
          "
        },
      })
      .request(PrepareRenameRequest {
        id: 2,
        uri: "file:///test.just",
        line: 1,
        character: 3,
      })
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": null
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn recipe_references() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo \"foo\"

          bar: foo
            echo \"bar\"

          alias baz := foo
          "
        },
      })
      .request(ReferencesRequest {
        id: 2,
        uri: "file:///test.just",
        line: 0,
        character: 1,
        include_declaration: true,
      })
      .response(ReferencesResponse {
        id: 2,
        locations: vec![
          Location {
            uri: "file:///test.just",
            start_line: 0,
            start_char: 0,
            end_line: 0,
            end_char: 3,
          },
          Location {
            uri: "file:///test.just",
            start_line: 3,
            start_char: 5,
            end_line: 3,
            end_char: 8,
          },
          Location {
            uri: "file:///test.just",
            start_line: 6,
            start_char: 13,
            end_line: 6,
            end_char: 16,
          },
        ],
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn rename_recipe() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo \"foo\"

          bar: foo
            echo \"bar\"

          alias baz := foo
          "
        },
      })
      .request(RenameRequest {
        id: 2,
        uri: "file:///test.just",
        line: 0,
        character: 1,
        new_name: "renamed",
      })
      .response(RenameResponse {
        id: 2,
        uri: "file:///test.just",
        edits: vec![
          Rename {
            start_line: 0,
            start_char: 0,
            end_line: 0,
            end_char: 3,
            new_text: "renamed",
          },
          Rename {
            start_line: 3,
            start_char: 5,
            end_line: 3,
            end_char: 8,
            new_text: "renamed",
          },
          Rename {
            start_line: 6,
            start_char: 13,
            end_line: 6,
            end_char: 16,
            new_text: "renamed",
          },
        ],
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn semantic_tokens_basic() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///test.just",
        text: indoc! {
          "
          foo:
            echo \"bar\"
          "
        },
      })
      .request(SemanticTokensRequest {
        id: 2,
        uri: "file:///test.just",
      })
      .response(SemanticTokensResponse {
        id: 2,
        data: vec![
          0, 0, 3, 6, 1, //
          0, 3, 1, 3, 0,
        ],
      })
      .run()
      .await
  }

  #[tokio::test]
  async fn shutdown() -> Result {
    Test::new()
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .request(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "shutdown",
      }))
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": null
      }))
      .run()
      .await
  }
}
