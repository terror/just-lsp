use super::*;

pub(crate) struct Server {
  client: Client,
  config: RwLock<Config>,
  diagnostics: Mutex<BTreeMap<lsp::Url, lsp::PublishDiagnosticsParams>>,
  executor: Executor,
  initialized: AtomicBool,
  workspace: RwLock<Workspace>,
}

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
    let executor = Executor::new(client.clone());

    Self {
      client,
      config: RwLock::new(Config::default()),
      diagnostics: Mutex::new(BTreeMap::new()),
      executor,
      initialized: AtomicBool::new(false),
      workspace: RwLock::new(Workspace::default()),
    }
  }

  async fn publish_diagnostics(&self) {
    if !self.initialized.load(Ordering::Relaxed) {
      return;
    }

    let mut previous = self.diagnostics.lock().await;

    let diagnostics = {
      let config = self.config.read().await;
      let workspace = self.workspace.read().await;

      workspace
        .diagnostics(Some(&config))
        .into_iter()
        .map(|(uri, diagnostics)| {
          let version = workspace
            .documents
            .get_open(&uri)
            .map(|document| document.version);

          let params = lsp::PublishDiagnosticsParams {
            uri: uri.clone(),
            diagnostics: diagnostics
              .into_iter()
              .map(lsp::Diagnostic::from)
              .collect(),
            version,
          };

          (uri, params)
        })
        .collect::<BTreeMap<_, _>>()
    };

    for uri in previous.keys() {
      if !diagnostics.contains_key(uri) {
        self
          .client
          .publish_diagnostics(uri.clone(), Vec::new(), None)
          .await;
      }
    }

    for (uri, params) in &diagnostics {
      if previous.get(uri) != Some(params) {
        self
          .client
          .publish_diagnostics(
            uri.clone(),
            params.diagnostics.clone(),
            params.version,
          )
          .await;
      }
    }

    *previous = diagnostics;
  }

  pub(crate) async fn run() -> Result {
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) = LspService::new(Server::new);

    tower_lsp::Server::new(stdin, stdout, socket)
      .serve(service)
      .await;

    Ok(())
  }

  async fn try_did_change(
    &self,
    params: lsp::DidChangeTextDocumentParams,
  ) -> Result {
    let uri = params.text_document.uri.clone();

    {
      let mut workspace = self.workspace.write().await;

      if !workspace.documents.is_open(&uri) {
        return Ok(());
      }

      let roots = workspace.affected_roots(&uri);

      workspace.documents.change(params)?;
      workspace.load_projects(roots.iter().cloned())?;
    }

    self.publish_diagnostics().await;

    Ok(())
  }

  async fn try_did_open(
    &self,
    params: lsp::DidOpenTextDocumentParams,
  ) -> Result {
    let uri = params.text_document.uri.clone();

    {
      let mut workspace = self.workspace.write().await;
      let mut roots = workspace.affected_roots(&uri);

      roots.insert(uri.clone());

      workspace.documents.open(params)?;
      workspace.load_projects(roots.iter().cloned())?;
    }

    self.publish_diagnostics().await;

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

    let diagnostics = workspace
      .document_diagnostics(&params.text_document.uri, Some(&config))
      .into_iter()
      .filter(|diagnostic| !diagnostic.quickfixes.is_empty())
      .collect::<Vec<_>>();

    let quickfixer = Quickfixer {
      diagnostics: &diagnostics,
      parameters: &params,
    };

    actions.extend(quickfixer.collect());

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

  async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
    if let Err(error) = self.try_did_change(params).await {
      self
        .client
        .log_message(lsp::MessageType::ERROR, error)
        .await;
    }
  }

  async fn did_close(&self, params: lsp::DidCloseTextDocumentParams) {
    let uri = params.text_document.uri.clone();

    {
      let mut workspace = self.workspace.write().await;
      let mut roots = workspace.affected_roots(&uri);

      let closed = workspace.documents.close(&params);

      if !closed {
        return;
      }

      if workspace.documents.get(&uri).is_none() {
        workspace.projects.remove(&uri);
        roots.remove(&uri);
      }

      if let Err(error) = workspace.load_projects(roots.iter().cloned()) {
        warn!(%error, "failed to rebuild affected projects");
      }
    }

    self.publish_diagnostics().await;
  }

  async fn did_open(&self, params: lsp::DidOpenTextDocumentParams) {
    if let Err(error) = self.try_did_open(params).await {
      self
        .client
        .log_message(lsp::MessageType::ERROR, error)
        .await;
    }
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
      capabilities: Self::capabilities(),
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

  async fn prepare_rename(
    &self,
    params: lsp::TextDocumentPositionParams,
  ) -> Result<Option<lsp::PrepareRenameResponse>, jsonrpc::Error> {
    let uri = &params.text_document.uri;

    let workspace = self.workspace.read().await;

    Ok(workspace.documents.get_open(uri).and_then(|document| {
      let resolver = Resolver::new(document);

      document
        .node_at_position(params.position)
        .filter(|node| node.kind() == "identifier")
        .filter(|identifier| {
          resolver
            .resolve_symbol(identifier)
            .is_some_and(|symbol| symbol.is_renameable())
        })
        .map(
          |identifier| lsp::PrepareRenameResponse::RangeWithPlaceholder {
            range: identifier.get_range(document),
            placeholder: document.get_node_text(&identifier),
          },
        )
    }))
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
      let resolver = Resolver::new(document);

      document
        .node_at_position(position)
        .filter(|node| node.kind() == "identifier")
        .filter(|identifier| {
          resolver
            .resolve_symbol(identifier)
            .is_some_and(|symbol| symbol.is_renameable())
        })
        .map(|identifier| {
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

  async fn shutdown(&self) -> Result<(), jsonrpc::Error> {
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::indoc,
    lsp::{notification, request},
    pretty_assertions::assert_eq,
    serde_json::json,
    tokio_stream::StreamExt,
    tower_lsp::ClientSocket,
    tower_test::mock::Spawn,
  };

  #[derive(Debug)]
  struct Test {
    messages: Vec<TestMessage>,
    service: Spawn<LspService<Server>>,
    socket: ClientSocket,
    tempdir: tempfile::TempDir,
  }

  #[derive(Debug)]
  struct TestMessage {
    expected: Option<jsonrpc::Response>,
    notifications: Vec<jsonrpc::Request>,
    request: jsonrpc::Request,
  }

  impl Test {
    fn change(self, path: &str, version: i32, text: &str) -> Self {
      self.edit(path, version, None, text)
    }

    fn client_notification<N: notification::Notification>(
      mut self,
      params: N::Params,
    ) -> Self {
      self
        .messages
        .last_mut()
        .unwrap()
        .notifications
        .push(Self::message(N::METHOD, params).finish());

      self
    }

    fn close(self, path: &str) -> Self {
      let uri = self.uri(path);

      self.notification::<notification::DidCloseTextDocument>(
        lsp::DidCloseTextDocumentParams {
          text_document: lsp::TextDocumentIdentifier::new(uri),
        },
      )
    }

    fn code_actions(
      self,
      path: &str,
      range: lsp::Range,
      actions: impl IntoIterator<Item = lsp::CodeActionOrCommand>,
    ) -> Self {
      let uri = self.uri(path);

      self.request::<request::CodeActionRequest>(
        lsp::CodeActionParams {
          text_document: lsp::TextDocumentIdentifier::new(uri),
          range,
          context: lsp::CodeActionContext::default(),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(actions.into_iter().collect())),
      )
    }

    fn definition(
      self,
      path: &str,
      position: lsp::Position,
      expected: Option<lsp::GotoDefinitionResponse>,
    ) -> Self {
      let uri = self.uri(path);

      self.request::<request::GotoDefinition>(
        lsp::GotoDefinitionParams {
          text_document_position_params: lsp::TextDocumentPositionParams::new(
            lsp::TextDocumentIdentifier::new(uri),
            position,
          ),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(expected),
      )
    }

    fn diagnostics(
      self,
      path: &str,
      version: Option<i32>,
      diagnostics: impl IntoIterator<Item = lsp::Diagnostic>,
    ) -> Self {
      let uri = self.uri(path);

      self.client_notification::<notification::PublishDiagnostics>(
        lsp::PublishDiagnosticsParams {
          uri,
          diagnostics: diagnostics.into_iter().collect(),
          version,
        },
      )
    }

    fn edit(
      self,
      path: &str,
      version: i32,
      range: impl Into<Option<lsp::Range>>,
      text: &str,
    ) -> Self {
      let uri = self.uri(path);

      self.notification::<notification::DidChangeTextDocument>(
        lsp::DidChangeTextDocumentParams {
          text_document: lsp::VersionedTextDocumentIdentifier::new(
            uri, version,
          ),
          content_changes: vec![lsp::TextDocumentContentChangeEvent {
            range: range.into(),
            range_length: None,
            text: text.into(),
          }],
        },
      )
    }

    fn file(self, path: &str, content: &str) -> Self {
      let path = self.tempdir.path().join(path);

      fs::create_dir_all(path.parent().unwrap()).unwrap();
      fs::write(path, content).unwrap();

      self
    }

    fn hover(
      self,
      path: &str,
      position: lsp::Position,
      expected: Option<lsp::Hover>,
    ) -> Self {
      let uri = self.uri(path);

      self.request::<request::HoverRequest>(
        lsp::HoverParams {
          text_document_position_params: lsp::TextDocumentPositionParams::new(
            lsp::TextDocumentIdentifier::new(uri),
            position,
          ),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
        },
        Ok(expected),
      )
    }

    fn initialize(self) -> Self {
      self.request::<request::Initialize>(
        lsp::InitializeParams::default(),
        Ok(lsp::InitializeResult {
          server_info: Some(lsp::ServerInfo {
            name: env!("CARGO_PKG_NAME").into(),
            version: Some(env!("CARGO_PKG_VERSION").into()),
          }),
          capabilities: Server::capabilities(),
        }),
      )
    }

    fn message(
      method: &'static str,
      params: impl Serialize,
    ) -> jsonrpc::RequestBuilder {
      let params = serde_json::to_value(params).unwrap();

      let request = jsonrpc::Request::build(method);

      if params.is_null() {
        request
      } else {
        request.params(params)
      }
    }

    fn new() -> Self {
      let (service, socket) = LspService::new(Server::new);

      Self {
        messages: Vec::new(),
        service: Spawn::new(service),
        socket,
        tempdir: tempfile::tempdir().unwrap(),
      }
    }

    fn notification<N: notification::Notification>(
      mut self,
      params: N::Params,
    ) -> Self {
      self.messages.push(TestMessage {
        expected: None,
        notifications: Vec::new(),
        request: Self::message(N::METHOD, params).finish(),
      });

      self
    }

    fn open(self, path: &str, text: &str) -> Self {
      let uri = self.uri(path);

      self.notification::<notification::DidOpenTextDocument>(
        lsp::DidOpenTextDocumentParams {
          text_document: lsp::TextDocumentItem::new(
            uri,
            "just".into(),
            1,
            text.into(),
          ),
        },
      )
    }

    fn quickfixes(
      self,
      path: &str,
      range: lsp::Range,
      quickfixes: impl IntoIterator<Item = Quickfix>,
    ) -> Self {
      let uri = self.uri(path);

      let actions = quickfixes.into_iter().map(|quickfix| {
        lsp::CodeActionOrCommand::CodeAction(lsp::CodeAction {
          title: quickfix.title().into(),
          kind: Some(lsp::CodeActionKind::QUICKFIX),
          edit: Some(lsp::WorkspaceEdit {
            changes: Some(HashMap::from([(
              uri.clone(),
              quickfix.edits().to_vec(),
            )])),
            ..Default::default()
          }),
          ..Default::default()
        })
      });

      self.code_actions(path, range, actions)
    }

    fn ready(self) -> Self {
      self
        .initialize()
        .notification::<notification::Initialized>(lsp::InitializedParams {})
        .client_notification::<notification::LogMessage>(
          lsp::LogMessageParams {
            typ: lsp::MessageType::INFO,
            message: format!("{} initialized", env!("CARGO_PKG_NAME")),
          },
        )
    }

    fn request<R: request::Request>(
      mut self,
      params: R::Params,
      expected: jsonrpc::Result<R::Result>,
    ) -> Self {
      let id = i64::try_from(self.messages.len()).unwrap();

      self.messages.push(TestMessage {
        expected: Some(jsonrpc::Response::from_parts(
          id.into(),
          expected.map(|result| serde_json::to_value(result).unwrap()),
        )),
        notifications: Vec::new(),
        request: Self::message(R::METHOD, params).id(id).finish(),
      });

      self
    }

    async fn run(mut self) -> Result {
      for TestMessage {
        expected,
        notifications,
        request,
      } in self.messages
      {
        let method = request.method().to_owned();

        let response = self.service.call(request);

        tokio::pin!(response);

        let mut actual = Vec::new();

        let response = loop {
          select! {
            biased;
            Some(notification) = self.socket.next() => actual.push(notification),
            response = &mut response => break response,
          }
        };

        loop {
          select! {
            biased;
            Some(notification) = self.socket.next() => actual.push(notification),
            () = async {} => break,
          }
        }

        assert_eq!(response?, expected, "{method}");

        assert_eq!(actual, notifications, "{method}");
      }

      Ok(())
    }

    fn uri(&self, path: &str) -> lsp::Url {
      path.parse().unwrap_or_else(|_| {
        lsp::Url::from_file_path(self.tempdir.path().join(path)).unwrap()
      })
    }
  }

  #[tokio::test]
  async fn closing_imported_buffer_restores_disk_project() -> Result {
    Test::new()
      .file("foo.just", "import 'bar.just'")
      .file("bar.just", "bar:")
      .initialize()
      .open("justfile", "import 'foo.just'\n\nfoo: bar")
      .open("foo.just", "")
      .hover("justfile", lsp::Position::new(2, 5), None)
      .close("foo.just")
      .hover(
        "justfile",
        lsp::Position::new(2, 5),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "bar:".into(),
          }),
          range: Some(lsp::Range::at(2, 5, 2, 8)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn code_action_deprecated_function_or_default_quickfix() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        "foo := env_var_or_default(\"BAR\", \"baz\")\n",
      )
      .quickfixes(
        "file:///test.just",
        lsp::Range::at(0, 10, 0, 10),
        [Quickfix::edit(
          "Replace `env_var_or_default` with `env`",
          lsp::Range::at(0, 7, 0, 25),
          "env",
        )],
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn code_action_deprecated_function_outside_range() -> Result {
    Test::new()
      .initialize()
      .open("file:///test.just", "foo := env_var(\"BAR\")\n")
      .code_actions("file:///test.just", lsp::Range::at(0, 0, 0, 3), [])
      .run()
      .await
  }

  #[tokio::test]
  async fn code_action_deprecated_function_quickfix() -> Result {
    Test::new()
      .initialize()
      .open("file:///test.just", "foo := env(\"BAR\")\n")
      .edit(
        "file:///test.just",
        2,
        lsp::Range::at(0, 7, 0, 10),
        "env_var",
      )
      .quickfixes(
        "file:///test.just",
        lsp::Range::at(0, 10, 0, 10),
        [Quickfix::edit(
          "Replace `env_var` with `env`",
          lsp::Range::at(0, 7, 0, 14),
          "env",
        )],
      )
      .edit("file:///test.just", 3, lsp::Range::at(0, 7, 0, 14), "env")
      .code_actions("file:///test.just", lsp::Range::at(0, 8, 0, 8), [])
      .run()
      .await
  }

  #[tokio::test]
  async fn code_action_empty_document() -> Result {
    Test::new()
      .initialize()
      .open("file:///empty.just", "")
      .code_actions("file:///empty.just", lsp::Range::at(0, 0, 0, 0), [])
      .run()
      .await
  }

  #[tokio::test]
  async fn code_action_shared_import_requires_matching_quickfixes() -> Result {
    let range = lsp::Range::at(0, 8, 0, 11);

    let diagnostic = lsp::Diagnostic::from(Diagnostic {
      id: "undefined-identifier".into(),
      ..Diagnostic::error(
        "Variable `bar` not found. Did you mean `baz`?",
        range,
      )
    });

    Test::new()
      .file("foo.just", "_foo := bar\n")
      .ready()
      .open("bar.just", "import 'foo.just'\nbar := 'foo'\n")
      .diagnostics("bar.just", Some(1), [])
      .diagnostics("foo.just", None, [])
      .open("baz.just", "import 'foo.just'\nexport baz := 'foo'\n")
      .diagnostics("baz.just", Some(1), [])
      .diagnostics("foo.just", None, [diagnostic.clone()])
      .open("foo.just", "_foo := bar\n")
      .diagnostics("foo.just", Some(1), [diagnostic])
      .code_actions("foo.just", range, [])
      .run()
      .await
  }

  #[tokio::test]
  async fn code_action_windows_shell_with_non_windows_attributes() -> Result {
    for (attributes, line) in [
      ("[linux]", 1),
      ("[unix]", 1),
      ("[linux, windows]", 1),
      ("[windows]\n[linux]", 2),
    ] {
      let range = lsp::Range::at(line, 4, line, 17);

      Test::new()
        .ready()
        .open(
          "foo.just",
          &format!("{attributes}\nset windows-shell := ['foo']\n"),
        )
        .diagnostics(
          "foo.just",
          Some(1),
          [Diagnostic {
            id: "deprecated-setting".into(),
            ..Diagnostic::warning(
              "`windows-shell` is deprecated, use `[windows]` attribute on `set shell` instead",
              range,
            )
          }
          .into()],
        )
        .code_actions("foo.just", range, [])
        .run()
        .await?;
    }

    Ok(())
  }

  #[tokio::test]
  async fn code_action_with_recipes() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo foo

          bar arg1 arg2='default':
            echo bar
          "
        },
      )
      .code_actions(
        "file:///test.just",
        lsp::Range::at(0, 0, 0, 0),
        [
          lsp::CodeActionOrCommand::CodeAction(lsp::CodeAction {
            title: "foo".into(),
            kind: Some(lsp::CodeActionKind::SOURCE),
            command: Some(lsp::Command {
              title: "foo".into(),
              command: Command::RunRecipe.to_string(),
              arguments: Some(vec![
                json!("foo"),
                json!("file:///test.just"),
                json!([]),
              ]),
            }),
            ..Default::default()
          }),
          lsp::CodeActionOrCommand::CodeAction(lsp::CodeAction {
            title: "bar".into(),
            kind: Some(lsp::CodeActionKind::SOURCE),
            command: Some(lsp::Command {
              title: "bar".into(),
              command: Command::RunRecipe.to_string(),
              arguments: Some(vec![
                json!("bar"),
                json!("file:///test.just"),
                json!(vec![
                  ParameterJson {
                    name: "arg1".into(),
                    default_value: None,
                  },
                  ParameterJson {
                    name: "arg2".into(),
                    default_value: Some("'default'".to_string()),
                  },
                ]),
              ]),
            }),
            ..Default::default()
          }),
        ],
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn code_lens_empty_document() -> Result {
    Test::new()
      .initialize()
      .open("file:///empty.just", "")
      .request::<request::CodeLensRequest>(
        lsp::CodeLensParams {
          text_document: lsp::TextDocumentIdentifier::new(
            "file:///empty.just".parse().unwrap(),
          ),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(vec![])),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn code_lens_with_recipes() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo foo

          bar arg1 arg2='default':
            echo bar
          "
        },
      )
      .request::<request::CodeLensRequest>(
        lsp::CodeLensParams {
          text_document: lsp::TextDocumentIdentifier::new(
            "file:///test.just".parse().unwrap(),
          ),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(vec![
          lsp::CodeLens {
            range: lsp::Range::at(0, 0, 0, 3),
            command: Some(lsp::Command {
              title: "Run".into(),
              command: "just-lsp.run_recipe".into(),
              arguments: Some(vec![
                json!("foo"),
                json!("file:///test.just"),
                json!([]),
              ]),
            }),
            data: None,
          },
          lsp::CodeLens {
            range: lsp::Range::at(3, 0, 3, 3),
            command: Some(lsp::Command {
              title: "Run".into(),
              command: "just-lsp.run_recipe".into(),
              arguments: Some(vec![
                json!("bar"),
                json!("file:///test.just"),
                json!([
                  { "name": "arg1", "default_value": null },
                  { "name": "arg2", "default_value": "'default'" }
                ]),
              ]),
            }),
            data: None,
          },
        ])),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn dependency_open_republishes_root_diagnostics() -> Result {
    Test::new()
      .file("foo.just", "")
      .ready()
      .open("justfile", "import 'foo.just'\n\nbar: foo")
      .diagnostics("foo.just", None, [])
      .diagnostics(
        "justfile",
        Some(1),
        [Diagnostic {
          id: "unresolved-dependency".into(),
          ..Diagnostic::error(
            "Recipe `foo` not found",
            lsp::Range::at(2, 5, 2, 8),
          )
        }
        .into()],
      )
      .open("foo.just", "foo:")
      .diagnostics("foo.just", Some(1), [])
      .diagnostics("justfile", Some(1), [])
      .run()
      .await
  }

  #[tokio::test]
  async fn diagnostics_clear_closed_projects() -> Result {
    Test::new()
      .file("foo.just", "foo: bar\n")
      .ready()
      .open("justfile", "import 'foo.just'\n")
      .diagnostics(
        "foo.just",
        None,
        [Diagnostic {
          id: "unresolved-dependency".into(),
          ..Diagnostic::error(
            "Recipe `bar` not found",
            lsp::Range::at(0, 5, 0, 8),
          )
        }
        .into()],
      )
      .diagnostics("justfile", Some(1), [])
      .close("justfile")
      .diagnostics("foo.just", None, [])
      .diagnostics("justfile", None, [])
      .run()
      .await
  }

  #[tokio::test]
  async fn diagnostics_clear_removed_imports() -> Result {
    Test::new()
      .file("foo.just", "foo: bar\n")
      .ready()
      .open("justfile", "import 'foo.just'\n")
      .diagnostics(
        "foo.just",
        None,
        [Diagnostic {
          id: "unresolved-dependency".into(),
          ..Diagnostic::error(
            "Recipe `bar` not found",
            lsp::Range::at(0, 5, 0, 8),
          )
        }
        .into()],
      )
      .diagnostics("justfile", Some(1), [])
      .change("justfile", 2, "")
      .diagnostics("foo.just", None, [])
      .diagnostics("justfile", Some(2), [])
      .run()
      .await
  }

  #[tokio::test]
  async fn diagnostics_closing_root_preserves_import_scope() -> Result {
    Test::new()
      .file("foo.just", "foo: bar\n")
      .file("justfile", "import 'foo.just'\nbar:\n")
      .ready()
      .open("justfile", "import 'foo.just'\nbar:\n")
      .diagnostics("foo.just", None, [])
      .diagnostics("justfile", Some(1), [])
      .open("foo.just", "foo: bar\n")
      .diagnostics("foo.just", Some(1), [])
      .close("justfile")
      .diagnostics("justfile", None, [])
      .change("foo.just", 2, "foo: bar\n")
      .diagnostics("foo.just", Some(2), [])
      .close("foo.just")
      .diagnostics("foo.just", None, [])
      .diagnostics("justfile", None, [])
      .run()
      .await
  }

  #[tokio::test]
  async fn diagnostics_closing_root_removes_import() -> Result {
    Test::new()
      .file("foo.just", "foo: bar\n")
      .file("justfile", "")
      .ready()
      .open("justfile", "import 'foo.just'\nbar:\n")
      .diagnostics("foo.just", None, [])
      .diagnostics("justfile", Some(1), [])
      .open("foo.just", "foo: bar\n")
      .diagnostics("foo.just", Some(1), [])
      .close("justfile")
      .diagnostics("justfile", None, [])
      .diagnostics(
        "foo.just",
        Some(1),
        [Diagnostic {
          id: "unresolved-dependency".into(),
          ..Diagnostic::error(
            "Recipe `bar` not found",
            lsp::Range::at(0, 5, 0, 8),
          )
        }
        .into()],
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn diagnostics_closing_root_restores_disk_scope() -> Result {
    Test::new()
      .file("foo.just", "foo: bar\n")
      .file("justfile", "import 'foo.just'\n")
      .ready()
      .open("justfile", "import 'foo.just'\nbar:\n")
      .diagnostics("foo.just", None, [])
      .diagnostics("justfile", Some(1), [])
      .open("foo.just", "foo: bar\n")
      .diagnostics("foo.just", Some(1), [])
      .close("justfile")
      .diagnostics(
        "foo.just",
        Some(1),
        [Diagnostic {
          id: "unresolved-dependency".into(),
          ..Diagnostic::error(
            "Recipe `bar` not found",
            lsp::Range::at(0, 5, 0, 8),
          )
        }
        .into()],
      )
      .diagnostics("justfile", None, [])
      .run()
      .await
  }

  #[tokio::test]
  async fn diagnostics_deduplicate_shared_import_quickfixes() -> Result {
    let range = lsp::Range::at(0, 4, 0, 17);

    let diagnostic = lsp::Diagnostic::from(Diagnostic {
      id: "deprecated-setting".into(),
      ..Diagnostic::warning(
        "`windows-shell` is deprecated, use `[windows]` attribute on `set shell` instead",
        range,
      )
    });

    Test::new()
      .file("foo.just", "set windows-shell := ['foo']\n")
      .ready()
      .open(
        "bar.just",
        "[windows]\nset shell := ['bar']\nimport 'foo.just'\n",
      )
      .diagnostics("bar.just", Some(1), [])
      .diagnostics("foo.just", None, [diagnostic.clone()])
      .open("baz.just", "import 'foo.just'\n")
      .diagnostics("baz.just", Some(1), [])
      .open("foo.just", "set windows-shell := ['foo']\n")
      .diagnostics("foo.just", Some(1), [diagnostic])
      .code_actions("foo.just", range, [])
      .change("bar.just", 2, "import 'foo.just'\n")
      .diagnostics("bar.just", Some(2), [])
      .quickfixes(
        "foo.just",
        range,
        [Quickfix::new(
          "Replace `windows-shell` with `[windows] set shell`",
          [
            lsp::TextEdit {
              range: lsp::Range::at(0, 4, 0, 17),
              new_text: "shell".into(),
            },
            lsp::TextEdit {
              range: lsp::Range::at(0, 0, 0, 0),
              new_text: "[windows]\n".into(),
            },
          ],
        )],
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn diagnostics_follow_import_conditions() -> Result {
    let disabled = if cfg!(windows) { "unix" } else { "windows" };

    Test::new()
      .file("foo.just", "foo: bar\n")
      .ready()
      .open("justfile", "import 'foo.just'\n")
      .diagnostics(
        "foo.just",
        None,
        [Diagnostic {
          id: "unresolved-dependency".into(),
          ..Diagnostic::error(
            "Recipe `bar` not found",
            lsp::Range::at(0, 5, 0, 8),
          )
        }
        .into()],
      )
      .diagnostics("justfile", Some(1), [])
      .change("justfile", 2, &format!("[{disabled}]\nimport 'foo.just'\n"))
      .diagnostics("foo.just", None, [])
      .diagnostics("justfile", Some(2), [])
      .change("justfile", 3, "import 'foo.just'\n")
      .diagnostics(
        "foo.just",
        None,
        [Diagnostic {
          id: "unresolved-dependency".into(),
          ..Diagnostic::error(
            "Recipe `bar` not found",
            lsp::Range::at(0, 5, 0, 8),
          )
        }
        .into()],
      )
      .diagnostics("justfile", Some(3), [])
      .run()
      .await
  }

  #[tokio::test]
  async fn diagnostics_handle_import_cycles() -> Result {
    Test::new()
      .file("foo.just", "import 'bar.just'\nfoo: bar\n")
      .file("bar.just", "import 'foo.just'\nbar:\n")
      .ready()
      .open("foo.just", "import 'bar.just'\nfoo: bar\n")
      .diagnostics("bar.just", None, [])
      .diagnostics("foo.just", Some(1), [])
      .open("bar.just", "import 'foo.just'\nbar:\n")
      .diagnostics("bar.just", Some(1), [])
      .run()
      .await
  }

  #[tokio::test]
  async fn diagnostics_include_unopened_nested_imports() -> Result {
    Test::new()
      .file("foo.just", "import 'bar.just'\n")
      .file("bar.just", "foo bar bar:\n  echo {{bar}}\n")
      .ready()
      .open("justfile", "import 'foo.just'\n")
      .diagnostics(
        "bar.just",
        None,
        [Diagnostic {
          id: "duplicate-recipe-parameter".into(),
          ..Diagnostic::error(
            "Duplicate parameter `bar`",
            lsp::Range::at(0, 8, 0, 11),
          )
        }
        .into()],
      )
      .diagnostics("foo.just", None, [])
      .diagnostics("justfile", Some(1), [])
      .run()
      .await
  }

  #[tokio::test]
  async fn diagnostics_keep_closed_imports() -> Result {
    Test::new()
      .file("foo.just", "foo: bar\n")
      .ready()
      .open("justfile", "import 'foo.just'\n")
      .diagnostics(
        "foo.just",
        None,
        [Diagnostic {
          id: "unresolved-dependency".into(),
          ..Diagnostic::error(
            "Recipe `bar` not found",
            lsp::Range::at(0, 5, 0, 8),
          )
        }
        .into()],
      )
      .diagnostics("justfile", Some(1), [])
      .open("foo.just", "foo:\n")
      .diagnostics("foo.just", Some(1), [])
      .close("foo.just")
      .diagnostics(
        "foo.just",
        None,
        [Diagnostic {
          id: "unresolved-dependency".into(),
          ..Diagnostic::error(
            "Recipe `bar` not found",
            lsp::Range::at(0, 5, 0, 8),
          )
        }
        .into()],
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn diagnostics_keep_shared_imports() -> Result {
    Test::new()
      .file("foo.just", "foo: bar\n")
      .ready()
      .open("bar.just", "import 'foo.just'\n")
      .diagnostics("bar.just", Some(1), [])
      .diagnostics(
        "foo.just",
        None,
        [Diagnostic {
          id: "unresolved-dependency".into(),
          ..Diagnostic::error(
            "Recipe `bar` not found",
            lsp::Range::at(0, 5, 0, 8),
          )
        }
        .into()],
      )
      .open("baz.just", "import 'foo.just'\n")
      .diagnostics("baz.just", Some(1), [])
      .close("bar.just")
      .diagnostics("bar.just", None, [])
      .close("baz.just")
      .diagnostics("baz.just", None, [])
      .diagnostics("foo.just", None, [])
      .run()
      .await
  }

  #[tokio::test]
  async fn diagnostics_open_import_uses_root_scope() -> Result {
    Test::new()
      .file("foo.just", "_foo() := bar\n")
      .ready()
      .open(
        "justfile",
        "set unstable\nbar := 'foo'\nimport 'foo.just'\n",
      )
      .diagnostics("foo.just", None, [])
      .diagnostics("justfile", Some(1), [])
      .open("foo.just", "_foo() := bar\n")
      .diagnostics("foo.just", Some(1), [])
      .hover(
        "foo.just",
        lsp::Position::new(0, 10),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "bar := 'foo'".into(),
          }),
          range: Some(lsp::Range::at(0, 10, 0, 13)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn diagnostics_refresh_imports_when_root_changes() -> Result {
    Test::new()
      .file("foo.just", "foo: bar\n")
      .ready()
      .open("justfile", "import 'foo.just'\nbar:\n")
      .diagnostics("foo.just", None, [])
      .diagnostics("justfile", Some(1), [])
      .change("justfile", 2, "import 'foo.just'\n")
      .diagnostics(
        "foo.just",
        None,
        [Diagnostic {
          id: "unresolved-dependency".into(),
          ..Diagnostic::error(
            "Recipe `bar` not found",
            lsp::Range::at(0, 5, 0, 8),
          )
        }
        .into()],
      )
      .diagnostics("justfile", Some(2), [])
      .run()
      .await
  }

  #[tokio::test]
  async fn diagnostics_root_open_replaces_import_scope() -> Result {
    Test::new()
      .file("foo.just", "foo: bar\n")
      .ready()
      .open("foo.just", "foo: bar\n")
      .diagnostics(
        "foo.just",
        Some(1),
        [Diagnostic {
          id: "unresolved-dependency".into(),
          ..Diagnostic::error(
            "Recipe `bar` not found",
            lsp::Range::at(0, 5, 0, 8),
          )
        }
        .into()],
      )
      .open("justfile", "import 'foo.just'\nbar:\n")
      .diagnostics("foo.just", Some(1), [])
      .diagnostics("justfile", Some(1), [])
      .run()
      .await
  }

  #[tokio::test]
  async fn did_change_handles_bare_carriage_returns() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///foo.just",
        "foo := '🧪\rbar'\r\n\nbaz:\n  echo {{ foo }}\n",
      )
      .edit("file:///foo.just", 2, lsp::Range::at(1, 0, 1, 3), "bar\r🧪")
      .edit("file:///foo.just", 3, lsp::Range::at(2, 0, 2, 2), "qux")
      .hover(
        "file:///foo.just",
        lsp::Position::new(5, 11),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "foo := '🧪\rbar\rqux'".into(),
          }),
          range: Some(lsp::Range::at(5, 10, 5, 13)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn did_change_handles_multibyte_characters() -> Result {
    Test::new()
      .initialize()
      .open("file:///foo.just", "# ─🧪\nfoo:\n  echo '─🧪'")
      .notification::<notification::DidChangeTextDocument>(
        lsp::DidChangeTextDocumentParams {
          text_document: lsp::VersionedTextDocumentIdentifier::new(
            "file:///foo.just".parse().unwrap(),
            2,
          ),
          content_changes: vec![
            lsp::TextDocumentContentChangeEvent {
              range: Some(lsp::Range::at(0, 2, 0, u32::MAX)),
              range_length: None,
              text: "bar".into(),
            },
            lsp::TextDocumentContentChangeEvent {
              range: Some(lsp::Range::at(2, 8, 2, u32::MAX)),
              range_length: None,
              text: "🧪bar'".into(),
            },
          ],
        },
      )
      .hover(
        "file:///foo.just",
        lsp::Position::new(1, 1),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "foo:\n  echo '🧪bar'".into(),
          }),
          range: Some(lsp::Range::at(1, 0, 1, 3)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn did_change_preserves_unicode_line_separators() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///foo.just",
        "foo:\n  echo 'foo\u{2028}bar'\n\nbaz: foo",
      )
      .edit(
        "file:///foo.just",
        2,
        lsp::Range::at(1, 12, 1, 15),
        "baz\u{2029}qux",
      )
      .hover(
        "file:///foo.just",
        lsp::Position::new(3, 6),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "foo:\n  echo 'foo\u{2028}baz\u{2029}qux'".into(),
          }),
          range: Some(lsp::Range::at(3, 5, 3, 8)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn did_change_updates_document() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo \"foo\"
          "
        },
      )
      .edit(
        "file:///test.just",
        2,
        lsp::Range::at(1, 7, 2, 0),
        "\"updated\"",
      )
      .hover(
        "file:///test.just",
        lsp::Position::new(0, 1),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "foo:\n  echo \"updated\"".into(),
          }),
          range: Some(lsp::Range::at(0, 0, 0, 3)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn did_change_without_open_document_is_ignored() -> Result {
    Test::new()
      .initialize()
      .edit(
        "file:///missing.just",
        2,
        lsp::Range::at(0, 0, 0, 0),
        "\"updated\"",
      )
      .open(
        "file:///missing.just",
        indoc! {
          "
          foo:
            echo \"foo\"

          bar: foo
            echo \"bar\"
          "
        },
      )
      .hover(
        "file:///missing.just",
        lsp::Position::new(3, 5),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "foo:\n  echo \"foo\"".into(),
          }),
          range: Some(lsp::Range::at(3, 5, 3, 8)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn document_highlight() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo \"foo\"

          bar: foo
            echo \"bar\"

          alias baz := foo
          "
        },
      )
      .request::<request::DocumentHighlightRequest>(
        lsp::DocumentHighlightParams {
          text_document_position_params: lsp::TextDocumentPositionParams::new(
            lsp::TextDocumentIdentifier::new(
              "file:///test.just".parse().unwrap(),
            ),
            lsp::Position::new(0, 1),
          ),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(vec![
          lsp::DocumentHighlight {
            range: lsp::Range::at(0, 0, 0, 3),
            kind: Some(lsp::DocumentHighlightKind::TEXT),
          },
          lsp::DocumentHighlight {
            range: lsp::Range::at(3, 5, 3, 8),
            kind: Some(lsp::DocumentHighlightKind::TEXT),
          },
          lsp::DocumentHighlight {
            range: lsp::Range::at(6, 13, 6, 16),
            kind: Some(lsp::DocumentHighlightKind::TEXT),
          },
        ])),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn document_link_empty_document() -> Result {
    Test::new()
      .initialize()
      .open("file:///test.just", "")
      .request::<request::DocumentLinkRequest>(
        lsp::DocumentLinkParams {
          text_document: lsp::TextDocumentIdentifier::new(
            "file:///test.just".parse().unwrap(),
          ),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(vec![])),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn document_link_ignores_unresolved_imports() -> Result {
    let tempdir = tempfile::tempdir()?;

    let root =
      lsp::Url::from_file_path(tempdir.path().join("justfile")).unwrap();

    Test::new()
      .initialize()
      .open(
        root.as_str(),
        "import? 'missing.just'\nimport x'dynamic.just'\n",
      )
      .request::<request::DocumentLinkRequest>(
        lsp::DocumentLinkParams {
          text_document: lsp::TextDocumentIdentifier::new(root.clone()),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(vec![])),
      )
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
      .initialize()
      .open(root_uri.as_str(), "import 'bar.just'\n")
      .request::<request::DocumentLinkRequest>(
        lsp::DocumentLinkParams {
          text_document: lsp::TextDocumentIdentifier::new(root_uri.clone()),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(vec![lsp::DocumentLink {
          range: lsp::Range::at(0, 7, 0, 17),
          target: Some(lsp::Url::from_file_path(&target).unwrap()),
          tooltip: Some(target.display().to_string()),
          data: None,
        }])),
      )
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
      .initialize()
      .open(justfile_uri, "mod bar x'baz.just'\n")
      .request::<request::DocumentLinkRequest>(
        lsp::DocumentLinkParams {
          text_document: lsp::TextDocumentIdentifier::new(
            justfile_uri.parse().unwrap(),
          ),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(vec![lsp::DocumentLink {
          range: lsp::Range::at(0, 8, 0, 19),
          target: Some(target_uri.parse().unwrap()),
          tooltip: Some(tooltip.into()),
          data: None,
        }])),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn document_symbol_empty_document() -> Result {
    Test::new()
      .initialize()
      .open("file:///empty.just", "")
      .request::<request::DocumentSymbolRequest>(
        lsp::DocumentSymbolParams {
          text_document: lsp::TextDocumentIdentifier::new(
            "file:///empty.just".parse().unwrap(),
          ),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(lsp::DocumentSymbolResponse::Nested(vec![]))),
      )
      .run()
      .await
  }

  #[tokio::test]
  #[allow(deprecated)]
  async fn document_symbol_with_alias() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo foo

          alias bar := foo
          "
        },
      )
      .request::<request::DocumentSymbolRequest>(
        lsp::DocumentSymbolParams {
          text_document: lsp::TextDocumentIdentifier::new(
            "file:///test.just".parse().unwrap(),
          ),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(lsp::DocumentSymbolResponse::Nested(vec![
          lsp::DocumentSymbol {
            name: "foo".into(),
            detail: None,
            kind: lsp::SymbolKind::FUNCTION,
            tags: None,
            deprecated: None,
            range: lsp::Range::at(0, 0, 3, 0),
            selection_range: lsp::Range::at(0, 0, 0, 3),
            children: None,
          },
          lsp::DocumentSymbol {
            name: "bar".into(),
            detail: Some("alias for foo".into()),
            kind: lsp::SymbolKind::FUNCTION,
            tags: None,
            deprecated: None,
            range: lsp::Range::at(3, 0, 3, 16),
            selection_range: lsp::Range::at(3, 6, 3, 9),
            children: None,
          },
        ]))),
      )
      .run()
      .await
  }

  #[tokio::test]
  #[allow(deprecated)]
  async fn document_symbol_with_recipes_and_variables() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          bar := 'baz'

          foo:
            echo foo
          "
        },
      )
      .request::<request::DocumentSymbolRequest>(
        lsp::DocumentSymbolParams {
          text_document: lsp::TextDocumentIdentifier::new(
            "file:///test.just".parse().unwrap(),
          ),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(lsp::DocumentSymbolResponse::Nested(vec![
          lsp::DocumentSymbol {
            name: "bar".into(),
            detail: None,
            kind: lsp::SymbolKind::VARIABLE,
            tags: None,
            deprecated: None,
            range: lsp::Range::at(0, 0, 1, 0),
            selection_range: lsp::Range::at(0, 0, 0, 3),
            children: None,
          },
          lsp::DocumentSymbol {
            name: "foo".into(),
            detail: None,
            kind: lsp::SymbolKind::FUNCTION,
            tags: None,
            deprecated: None,
            range: lsp::Range::at(2, 0, 4, 0),
            selection_range: lsp::Range::at(2, 0, 2, 3),
            children: None,
          },
        ]))),
      )
      .run()
      .await
  }

  #[tokio::test]
  #[allow(deprecated)]
  async fn document_symbol_with_setting() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          set export := true

          foo:
            echo foo
          "
        },
      )
      .request::<request::DocumentSymbolRequest>(
        lsp::DocumentSymbolParams {
          text_document: lsp::TextDocumentIdentifier::new(
            "file:///test.just".parse().unwrap(),
          ),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(lsp::DocumentSymbolResponse::Nested(vec![
          lsp::DocumentSymbol {
            name: "export".into(),
            detail: Some("boolean".into()),
            kind: lsp::SymbolKind::PROPERTY,
            tags: None,
            deprecated: None,
            range: lsp::Range::at(0, 0, 1, 0),
            selection_range: lsp::Range::at(0, 0, 1, 0),
            children: None,
          },
          lsp::DocumentSymbol {
            name: "foo".into(),
            detail: None,
            kind: lsp::SymbolKind::FUNCTION,
            tags: None,
            deprecated: None,
            range: lsp::Range::at(2, 0, 4, 0),
            selection_range: lsp::Range::at(2, 0, 2, 3),
            children: None,
          },
        ]))),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn execute_command_rejects_non_file_uri() -> Result {
    Test::new()
      .initialize()
      .request::<request::ExecuteCommand>(
        lsp::ExecuteCommandParams {
          command: Command::RunRecipe.to_string(),
          arguments: vec![json!("foo"), json!("untitled:foo.just"), json!([])],
          ..Default::default()
        },
        Ok(None),
      )
      .client_notification::<notification::ShowMessage>(
        lsp::ShowMessageParams {
          typ: lsp::MessageType::ERROR,
          message: "document URI `untitled:foo.just` is not a file URI".into(),
        },
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn folding_range() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo \"foo\"
            echo \"another line\"

          bar:
            echo \"bar\"
          "
        },
      )
      .request::<request::FoldingRangeRequest>(
        lsp::FoldingRangeParams {
          text_document: lsp::TextDocumentIdentifier::new(
            "file:///test.just".parse().unwrap(),
          ),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(vec![
          lsp::FoldingRange {
            start_line: 0,
            end_line: 3,
            kind: Some(lsp::FoldingRangeKind::Region),
            ..Default::default()
          },
          lsp::FoldingRange {
            start_line: 4,
            end_line: 5,
            kind: Some(lsp::FoldingRangeKind::Region),
            ..Default::default()
          },
        ])),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn goto_import_definition() -> Result {
    let test = Test::new().file("foo.just", "foo:");
    let target = test.uri("foo.just");

    test
      .initialize()
      .open("justfile", "import 'foo.just'")
      .definition(
        "justfile",
        lsp::Position::new(0, 9),
        Some(lsp::GotoDefinitionResponse::Scalar(lsp::Location {
          uri: target,
          range: lsp::Range::at(0, 0, 0, 0),
        })),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn goto_recipe_definition_from_dependency() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo \"foo\"

          bar: foo
            echo \"bar\"
          "
        },
      )
      .definition(
        "file:///test.just",
        lsp::Position::new(3, 5),
        Some(lsp::GotoDefinitionResponse::Scalar(lsp::Location {
          uri: "file:///test.just".parse().unwrap(),
          range: lsp::Range::at(0, 0, 3, 0),
        })),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_attribute() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          [no-cd]
          foo:
            echo \"foo\"
          "
        },
      )
      .hover(
        "file:///test.just",
        lsp::Position::new(0, 3),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
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
            .into(),
          }),
          range: Some(lsp::Range::at(0, 1, 0, 6)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_builtin_function() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo {{arch()}}
          "
        },
      )
      .hover(
        "file:///test.just",
        lsp::Position::new(1, 11),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
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
            .into(),
          }),
          range: Some(lsp::Range::at(1, 9, 1, 13)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_constant() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo {{ HEX }}

          bar: foo
            echo \"bar\"
          "
        },
      )
      .hover(
        "file:///test.just",
        lsp::Position::new(1, 12),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::Markdown,
            value: indoc! {
              "
              Lowercase hexadecimal digit string: `\"0123456789abcdef\"`.

              Useful as the alphabet argument to `choose()` for generating
              random hex strings.

              ```just
              token := choose('32', HEX)
              ```
              "
            }
            .into(),
          }),
          range: Some(lsp::Range::at(1, 10, 1, 13)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_local_parameter() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          bar arg='cooler':
            echo {{ arg }}

          foo arg='cool':
            echo {{ arg }}
          "
        },
      )
      .hover(
        "file:///test.just",
        lsp::Position::new(4, 11),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "arg='cool'".into(),
          }),
          range: Some(lsp::Range::at(4, 10, 4, 13)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_prioritize_recipe_parameter_over_variable_in_interpolation()
  -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          arg := 'wow'

          foo arg='cool':
            echo {{ arg }}
          "
        },
      )
      .hover(
        "file:///test.just",
        lsp::Position::new(3, 11),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "arg='cool'".into(),
          }),
          range: Some(lsp::Range::at(3, 10, 3, 13)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_recipe() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo \"foo\"

          bar: foo
            echo \"bar\"
          "
        },
      )
      .hover(
        "file:///test.just",
        lsp::Position::new(3, 5),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "foo:\n  echo \"foo\"".into(),
          }),
          range: Some(lsp::Range::at(3, 5, 3, 8)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_recipe_parameter_in_interpolation() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo arg='cool':
            echo {{ arg }}
          "
        },
      )
      .hover(
        "file:///test.just",
        lsp::Position::new(1, 11),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "arg='cool'".into(),
          }),
          range: Some(lsp::Range::at(1, 10, 1, 13)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_same_named_recipes_and_functions() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          arch:
            echo \"foo\"

          bar: arch
            echo {{ arch() }}
          "
        },
      )
      .hover(
        "file:///test.just",
        lsp::Position::new(3, 5),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "arch:\n  echo \"foo\"".into(),
          }),
          range: Some(lsp::Range::at(3, 5, 3, 9)),
        }),
      )
      .hover(
        "file:///test.just",
        lsp::Position::new(4, 11),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
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
            .into(),
          }),
          range: Some(lsp::Range::at(4, 10, 4, 14)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_setting() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          set export := true

          foo:
            echo \"foo\"
          "
        },
      )
      .hover(
        "file:///test.just",
        lsp::Position::new(0, 4),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
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
            .into(),
          }),
          range: Some(lsp::Range::at(0, 4, 0, 10)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn hover_variable_in_interpolation() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo := 'foo'

          foo:
            echo {{ foo }}
          "
        },
      )
      .hover(
        "file:///test.just",
        lsp::Position::new(3, 11),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "foo := 'foo'".into(),
          }),
          range: Some(lsp::Range::at(3, 10, 3, 13)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn imported_symbol_navigation() -> Result {
    let test = Test::new().file(
      "foo.just",
      indoc! {
        "
        qux:
          echo foo

        bar := 'baz'

        qux() := 'quux'
        "
      },
    );

    let target = test.uri("foo.just");

    test
      .initialize()
      .open(
        "justfile",
        indoc! {
          "
          import 'foo.just'

          foo bar='local': qux
            echo {{ bar }}
            echo {{ qux() }}

          baz:
            echo {{ bar }}
          "
        },
      )
      .definition(
        "justfile",
        lsp::Position::new(2, 18),
        Some(lsp::GotoDefinitionResponse::Scalar(lsp::Location {
          uri: target.clone(),
          range: lsp::Range::at(0, 0, 3, 0),
        })),
      )
      .definition(
        "justfile",
        lsp::Position::new(4, 11),
        Some(lsp::GotoDefinitionResponse::Scalar(lsp::Location {
          uri: target.clone(),
          range: lsp::Range::at(5, 0, 5, 3),
        })),
      )
      .definition(
        "justfile",
        lsp::Position::new(7, 11),
        Some(lsp::GotoDefinitionResponse::Scalar(lsp::Location {
          uri: target,
          range: lsp::Range::at(3, 0, 4, 0),
        })),
      )
      .hover(
        "justfile",
        lsp::Position::new(2, 18),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "qux:\n  echo foo".into(),
          }),
          range: Some(lsp::Range::at(2, 17, 2, 20)),
        }),
      )
      .hover(
        "justfile",
        lsp::Position::new(4, 11),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "qux() := 'quux'".into(),
          }),
          range: Some(lsp::Range::at(4, 10, 4, 13)),
        }),
      )
      .hover(
        "justfile",
        lsp::Position::new(7, 11),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "bar := 'baz'".into(),
          }),
          range: Some(lsp::Range::at(7, 10, 7, 13)),
        }),
      )
      .hover(
        "justfile",
        lsp::Position::new(3, 11),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "bar='local'".into(),
          }),
          range: Some(lsp::Range::at(3, 10, 3, 13)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn imported_symbol_navigation_rebuilds_affected_roots() -> Result {
    Test::new()
      .file("qux.just", "qux:")
      .initialize()
      .open("foo.just", "import 'baz.just'\n\nfoo: qux")
      .open("bar.just", "import 'baz.just'\n\nbar: qux")
      .open("baz.just", "")
      .hover("foo.just", lsp::Position::new(2, 5), None)
      .change("baz.just", 2, "import 'qux.just'")
      .hover(
        "foo.just",
        lsp::Position::new(2, 5),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "qux:".into(),
          }),
          range: Some(lsp::Range::at(2, 5, 2, 8)),
        }),
      )
      .hover(
        "bar.just",
        lsp::Position::new(2, 5),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "qux:".into(),
          }),
          range: Some(lsp::Range::at(2, 5, 2, 8)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn imported_symbol_navigation_uses_open_buffer() -> Result {
    let test = Test::new().file("foo.just", "foo:\n  echo disk");

    let target = test.uri("foo.just");

    test
      .initialize()
      .open("justfile", "import 'foo.just'\n\nbar: foo")
      .open("foo.just", "\nfoo:\n  echo buffer")
      .definition(
        "justfile",
        lsp::Position::new(2, 5),
        Some(lsp::GotoDefinitionResponse::Scalar(lsp::Location {
          uri: target,
          range: lsp::Range::at(1, 0, 2, 13),
        })),
      )
      .hover(
        "justfile",
        lsp::Position::new(2, 5),
        Some(lsp::Hover {
          contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::PlainText,
            value: "foo:\n  echo buffer".into(),
          }),
          range: Some(lsp::Range::at(2, 5, 2, 8)),
        }),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn initialize() -> Result {
    Test::new().initialize().run().await
  }

  #[tokio::test]
  async fn initialize_once() -> Result {
    Test::new()
      .initialize()
      .request::<request::Initialize>(
        lsp::InitializeParams::default(),
        Err(jsonrpc::Error::invalid_request()),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn prepare_rename_builtin_function() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo {{arch()}}
          "
        },
      )
      .request::<request::PrepareRenameRequest>(
        lsp::TextDocumentPositionParams::new(
          lsp::TextDocumentIdentifier::new(
            "file:///test.just".parse().unwrap(),
          ),
          lsp::Position::new(1, 11),
        ),
        Ok(None),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn prepare_rename_identifier() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo \"foo\"
          "
        },
      )
      .request::<request::PrepareRenameRequest>(
        lsp::TextDocumentPositionParams::new(
          lsp::TextDocumentIdentifier::new(
            "file:///test.just".parse().unwrap(),
          ),
          lsp::Position::new(0, 1),
        ),
        Ok(Some(lsp::PrepareRenameResponse::RangeWithPlaceholder {
          range: lsp::Range::at(0, 0, 0, 3),
          placeholder: "foo".into(),
        })),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn prepare_rename_non_identifier() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo \"foo\"
          "
        },
      )
      .request::<request::PrepareRenameRequest>(
        lsp::TextDocumentPositionParams::new(
          lsp::TextDocumentIdentifier::new(
            "file:///test.just".parse().unwrap(),
          ),
          lsp::Position::new(1, 3),
        ),
        Ok(None),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn prepare_rename_undefined() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo {{ missing }}
          "
        },
      )
      .request::<request::PrepareRenameRequest>(
        lsp::TextDocumentPositionParams::new(
          lsp::TextDocumentIdentifier::new(
            "file:///test.just".parse().unwrap(),
          ),
          lsp::Position::new(1, 13),
        ),
        Ok(None),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn prepare_rename_variable() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          x := '1'

          foo:
            echo {{ x }}
          "
        },
      )
      .request::<request::PrepareRenameRequest>(
        lsp::TextDocumentPositionParams::new(
          lsp::TextDocumentIdentifier::new(
            "file:///test.just".parse().unwrap(),
          ),
          lsp::Position::new(0, 0),
        ),
        Ok(Some(lsp::PrepareRenameResponse::RangeWithPlaceholder {
          range: lsp::Range::at(0, 0, 0, 1),
          placeholder: "x".into(),
        })),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn recipe_references() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo \"foo\"

          bar: foo
            echo \"bar\"

          alias baz := foo
          "
        },
      )
      .request::<request::References>(
        lsp::ReferenceParams {
          text_document_position: lsp::TextDocumentPositionParams::new(
            lsp::TextDocumentIdentifier::new(
              "file:///test.just".parse().unwrap(),
            ),
            lsp::Position::new(0, 1),
          ),
          context: lsp::ReferenceContext {
            include_declaration: true,
          },
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(vec![
          lsp::Location {
            uri: "file:///test.just".parse().unwrap(),
            range: lsp::Range::at(0, 0, 0, 3),
          },
          lsp::Location {
            uri: "file:///test.just".parse().unwrap(),
            range: lsp::Range::at(3, 5, 3, 8),
          },
          lsp::Location {
            uri: "file:///test.just".parse().unwrap(),
            range: lsp::Range::at(6, 13, 6, 16),
          },
        ])),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn rename_builtin() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo {{arch()}}
          "
        },
      )
      .request::<request::Rename>(
        lsp::RenameParams {
          text_document_position: lsp::TextDocumentPositionParams::new(
            lsp::TextDocumentIdentifier::new(
              "file:///test.just".parse().unwrap(),
            ),
            lsp::Position::new(1, 11),
          ),
          new_name: "cpu".into(),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
        },
        Ok(None),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn rename_parameter_default_symbols() -> Result {
    async fn case(ranges: &[lsp::Range]) -> Result {
      for range in ranges {
        Test::new()
          .initialize()
          .open(
            "file:///foo.just",
            indoc! {
              "
              foo := 'bar'
              baz foo=foo bar=foo:
                echo {{ foo }}
              "
            },
          )
          .request::<request::Rename>(
            lsp::RenameParams {
              text_document_position: lsp::TextDocumentPositionParams::new(
                lsp::TextDocumentIdentifier::new(
                  "file:///foo.just".parse().unwrap(),
                ),
                range.start,
              ),
              new_name: "qux".into(),
              work_done_progress_params: lsp::WorkDoneProgressParams::default(),
            },
            Ok(Some(lsp::WorkspaceEdit {
              changes: Some(HashMap::from([(
                "file:///foo.just".parse().unwrap(),
                ranges
                  .iter()
                  .map(|range| lsp::TextEdit {
                    range: *range,
                    new_text: "qux".into(),
                  })
                  .collect(),
              )])),
              ..Default::default()
            })),
          )
          .run()
          .await?;
      }

      Ok(())
    }

    case(&[
      lsp::Range::at(1, 4, 1, 7),
      lsp::Range::at(1, 16, 1, 19),
      lsp::Range::at(2, 10, 2, 13),
    ])
    .await?;

    case(&[lsp::Range::at(0, 0, 0, 3), lsp::Range::at(1, 8, 1, 11)]).await
  }

  #[tokio::test]
  async fn rename_recipe() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo \"foo\"

          bar: foo
            echo \"bar\"

          alias baz := foo
          "
        },
      )
      .request::<request::Rename>(
        lsp::RenameParams {
          text_document_position: lsp::TextDocumentPositionParams::new(
            lsp::TextDocumentIdentifier::new(
              "file:///test.just".parse().unwrap(),
            ),
            lsp::Position::new(0, 1),
          ),
          new_name: "renamed".into(),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
        },
        Ok(Some(lsp::WorkspaceEdit {
          changes: Some(HashMap::from([(
            "file:///test.just".parse().unwrap(),
            vec![
              lsp::TextEdit {
                range: lsp::Range::at(0, 0, 0, 3),
                new_text: "renamed".into(),
              },
              lsp::TextEdit {
                range: lsp::Range::at(3, 5, 3, 8),
                new_text: "renamed".into(),
              },
              lsp::TextEdit {
                range: lsp::Range::at(6, 13, 6, 16),
                new_text: "renamed".into(),
              },
            ],
          )])),
          ..Default::default()
        })),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn rename_undefined() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo {{ missing }}
          "
        },
      )
      .request::<request::Rename>(
        lsp::RenameParams {
          text_document_position: lsp::TextDocumentPositionParams::new(
            lsp::TextDocumentIdentifier::new(
              "file:///test.just".parse().unwrap(),
            ),
            lsp::Position::new(1, 13),
          ),
          new_name: "defined".into(),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
        },
        Ok(None),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn semantic_tokens_basic() -> Result {
    Test::new()
      .initialize()
      .open(
        "file:///test.just",
        indoc! {
          "
          foo:
            echo \"bar\"
          "
        },
      )
      .request::<request::SemanticTokensFullRequest>(
        lsp::SemanticTokensParams {
          text_document: lsp::TextDocumentIdentifier::new(
            "file:///test.just".parse().unwrap(),
          ),
          work_done_progress_params: lsp::WorkDoneProgressParams::default(),
          partial_result_params: lsp::PartialResultParams::default(),
        },
        Ok(Some(lsp::SemanticTokensResult::Tokens(
          lsp::SemanticTokens {
            result_id: None,
            data: vec![
              lsp::SemanticToken {
                delta_line: 0,
                delta_start: 0,
                length: 3,
                token_type: 6,
                token_modifiers_bitset: 1,
              },
              lsp::SemanticToken {
                delta_line: 0,
                delta_start: 3,
                length: 1,
                token_type: 3,
                token_modifiers_bitset: 0,
              },
            ],
          },
        ))),
      )
      .run()
      .await
  }

  #[tokio::test]
  async fn shutdown() -> Result {
    Test::new()
      .initialize()
      .request::<request::Shutdown>((), Ok(()))
      .run()
      .await
  }
}
