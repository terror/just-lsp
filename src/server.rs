use super::*;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncRead, AsyncWriteExt, ReadBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Clone)]
pub(crate) struct DebugLogger {
  log_path: Option<PathBuf>,
}

impl DebugLogger {
  fn new(log_path: Option<PathBuf>) -> Self {
    Self { log_path }
  }
  
  async fn log(&self, message: &str) {
    if let Some(path) = &self.log_path {
      let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
      let log_line = format!("[{}] {}\n", timestamp, message);
      
      if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
      {
        let _ = file.write_all(log_line.as_bytes()).await;
        let _ = file.flush().await;
      }
    }
  }
}

struct LoggingReader<R> {
  inner: R,
  logger: DebugLogger,
}

impl<R> LoggingReader<R> {
  fn new(inner: R, logger: DebugLogger) -> Self {
    Self { inner, logger }
  }
}

impl<R: AsyncRead + Unpin> AsyncRead for LoggingReader<R> {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut ReadBuf<'_>,
  ) -> Poll<tokio::io::Result<()>> {
    let before_len = buf.filled().len();
    let result = Pin::new(&mut self.inner).poll_read(cx, buf);
    
    if let Poll::Ready(Ok(())) = &result {
      let new_data = &buf.filled()[before_len..];
      if !new_data.is_empty() {
        let logger = self.logger.clone();
        let data_str = String::from_utf8_lossy(new_data).to_string();
        tokio::spawn(async move {
          logger.log(&format!("RAW_INPUT: {}", data_str.trim())).await;
        });
      }
    }
    
    result
  }
}

#[derive(Debug)]
pub(crate) struct Server(Arc<tokio::sync::Mutex<Inner>>);

impl Server {
  #[allow(dead_code)]
  pub fn new(client: Client) -> Self {
    Self(Arc::new(tokio::sync::Mutex::new(Inner::new(client, DebugLogger::new(None)))))
  }
  
  pub fn new_with_logger(client: Client, logger: DebugLogger) -> Self {
    Self(Arc::new(tokio::sync::Mutex::new(Inner::new(client, logger))))
  }

  pub async fn run(_raw_mode: bool, log_path: Option<std::path::PathBuf>) -> Result {
    let debug_logger = DebugLogger::new(log_path);
    
    debug_logger.log("SERVER: Starting just-lsp server").await;
    
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    // Wrap stdin to log all incoming data
    let logging_stdin = LoggingReader::new(stdin, debug_logger.clone());

    let logger_clone = debug_logger.clone();
    let (service, socket) = LspService::new(move |client| {
      let logger = logger_clone.clone();
      tokio::spawn(async move {
        logger.log("SERVER: Creating new server instance for client").await;
      });
      Server::new_with_logger(client, logger_clone.clone())
    });

    debug_logger.log("SERVER: LSP service created, starting to serve requests").await;

    tower_lsp::Server::new(logging_stdin, stdout, socket)
      .serve(service)
      .await;

    debug_logger.log("SERVER: Just-lsp server stopped").await;
    Ok(())
  }

  pub(crate) fn capabilities() -> lsp::ServerCapabilities {
    lsp::ServerCapabilities {
      completion_provider: Some(lsp::CompletionOptions {
        ..Default::default()
      }),
      code_action_provider: Some(lsp::CodeActionProviderCapability::Simple(
        true,
      )),
      definition_provider: Some(lsp::OneOf::Left(true)),
      document_formatting_provider: Some(lsp::OneOf::Left(true)),
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
      rename_provider: Some(lsp::OneOf::Left(true)),
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
}

#[tower_lsp::async_trait]
impl LanguageServer for Server {
  async fn code_action(
    &self,
    params: lsp::CodeActionParams,
  ) -> Result<Option<lsp::CodeActionResponse>, jsonrpc::Error> {
    self.0.lock().await.code_action(params).await
  }

  async fn completion(
    &self,
    params: lsp::CompletionParams,
  ) -> Result<Option<lsp::CompletionResponse>, jsonrpc::Error> {
    self.0.lock().await.completion(params).await
  }

  async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
    let mut inner = self.0.lock().await;

    if let Err(error) = inner.did_change(params).await {
      inner
        .client
        .log_message(lsp::MessageType::ERROR, error)
        .await;
    }
  }

  async fn did_close(&self, params: lsp::DidCloseTextDocumentParams) {
    self.0.lock().await.did_close(params).await
  }

  async fn did_open(&self, params: lsp::DidOpenTextDocumentParams) {
    let mut inner = self.0.lock().await;

    if let Err(error) = inner.did_open(params).await {
      inner
        .client
        .log_message(lsp::MessageType::ERROR, error)
        .await;
    }
  }

  async fn document_highlight(
    &self,
    params: lsp::DocumentHighlightParams,
  ) -> Result<Option<Vec<lsp::DocumentHighlight>>, jsonrpc::Error> {
    self.0.lock().await.document_highlight(params).await
  }

  async fn execute_command(
    &self,
    params: lsp::ExecuteCommandParams,
  ) -> Result<Option<serde_json::Value>, jsonrpc::Error> {
    self.0.lock().await.execute_command(params).await
  }

  async fn folding_range(
    &self,
    params: lsp::FoldingRangeParams,
  ) -> Result<Option<Vec<lsp::FoldingRange>>, jsonrpc::Error> {
    self.0.lock().await.folding_range(params).await
  }

  async fn formatting(
    &self,
    params: lsp::DocumentFormattingParams,
  ) -> Result<Option<Vec<lsp::TextEdit>>, jsonrpc::Error> {
    self.0.lock().await.formatting(params).await
  }

  async fn goto_definition(
    &self,
    params: lsp::GotoDefinitionParams,
  ) -> Result<Option<lsp::GotoDefinitionResponse>, jsonrpc::Error> {
    self.0.lock().await.goto_definition(params).await
  }

  async fn hover(
    &self,
    params: lsp::HoverParams,
  ) -> Result<Option<lsp::Hover>, jsonrpc::Error> {
    self.0.lock().await.hover(params).await
  }

  async fn initialize(
    &self,
    params: lsp::InitializeParams,
  ) -> Result<lsp::InitializeResult, jsonrpc::Error> {
    self.0.lock().await.initialize(params).await
  }

  async fn initialized(&self, params: lsp::InitializedParams) {
    self.0.lock().await.initialized(params).await
  }

  async fn references(
    &self,
    params: lsp::ReferenceParams,
  ) -> Result<Option<Vec<lsp::Location>>, jsonrpc::Error> {
    self.0.lock().await.references(params).await
  }

  async fn rename(
    &self,
    params: lsp::RenameParams,
  ) -> Result<Option<lsp::WorkspaceEdit>, jsonrpc::Error> {
    self.0.lock().await.rename(params).await
  }

  async fn shutdown(&self) -> Result<(), jsonrpc::Error> {
    self.0.lock().await.shutdown().await
  }
}

#[derive(Debug)]
pub(crate) struct Inner {
  client: Client,
  documents: BTreeMap<lsp::Url, Document>,
  initialized: bool,
  shutdown_received: bool,
  debug_logger: DebugLogger,
}

impl Inner {
  fn new(client: Client, debug_logger: DebugLogger) -> Self {
    Self {
      client,
      documents: BTreeMap::new(),
      initialized: false,
      shutdown_received: false,
      debug_logger,
    }
  }

  async fn code_action(
    &self,
    params: lsp::CodeActionParams,
  ) -> Result<Option<lsp::CodeActionResponse>, jsonrpc::Error> {
    let uri = &params.text_document.uri;

    if let Some(document) = self.documents.get(uri) {
      let mut actions = Vec::new();

      for recipe in document.get_recipes() {
        let parameters = recipe
          .parameters
          .into_iter()
          .map(ParameterJson::from)
          .collect::<Vec<ParameterJson>>();

        let recipe_name = serde_json::to_value(&recipe.name)
          .map_err(|_| jsonrpc::Error::parse_error())?;

        let uri = serde_json::to_value(uri)
          .map_err(|_| jsonrpc::Error::parse_error())?;

        let parameters = serde_json::to_value(parameters)
          .map_err(|_| jsonrpc::Error::parse_error())?;

        actions.push(lsp::CodeActionOrCommand::CodeAction(lsp::CodeAction {
          title: recipe.name.clone(),
          kind: Some(lsp::CodeActionKind::SOURCE),
          command: Some(lsp::Command {
            title: recipe.name.clone(),
            command: Command::RunRecipe.to_string(),
            arguments: Some(vec![recipe_name, uri, parameters]),
          }),
          ..Default::default()
        }));
      }

      return Ok(Some(actions));
    }

    Ok(None)
  }

  async fn completion(
    &self,
    params: lsp::CompletionParams,
  ) -> Result<Option<lsp::CompletionResponse>, jsonrpc::Error> {
    let uri = params.text_document_position.text_document.uri;

    if let Some(document) = self.documents.get(&uri) {
      let mut completion_items = Vec::new();

      let recipes = document.get_recipes();

      for recipe in recipes {
        completion_items.push(lsp::CompletionItem {
          label: recipe.name.clone(),
          kind: Some(lsp::CompletionItemKind::FUNCTION),
          documentation: Some(lsp::Documentation::MarkupContent(
            lsp::MarkupContent {
              kind: lsp::MarkupKind::PlainText,
              value: recipe.content,
            },
          )),
          insert_text: Some(recipe.name),
          insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
          ..Default::default()
        });
      }

      let variables = document.get_variables();

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

      for builtin in builtins::BUILTINS {
        completion_items.push(builtin.completion_item());
      }

      return Ok(Some(lsp::CompletionResponse::Array(completion_items)));
    }

    Ok(None)
  }

  async fn did_change(
    &mut self,
    params: lsp::DidChangeTextDocumentParams,
  ) -> Result {
    let uri = params.text_document.uri.clone();

    if let Some(document) = self.documents.get_mut(&params.text_document.uri) {
      document.apply_change(params)?;
      self.publish_diagnostics(&uri).await;
    }

    Ok(())
  }

  async fn did_close(&mut self, params: lsp::DidCloseTextDocumentParams) {
    let uri = &params.text_document.uri;

    if self.documents.contains_key(uri) {
      self.documents.remove(uri);

      self
        .client
        .publish_diagnostics(uri.clone(), vec![], None)
        .await;
    }
  }

  async fn did_open(
    &mut self,
    params: lsp::DidOpenTextDocumentParams,
  ) -> Result {
    self.debug_logger.log(&format!("RECEIVED: didOpen for document: {}", params.text_document.uri)).await;
    
    let uri = params.text_document.uri.clone();

    self.documents.insert(
      params.text_document.uri.to_owned(),
      Document::try_from(params)?,
    );

    self.publish_diagnostics(&uri).await;

    self.debug_logger.log(&format!("PROCESSED: didOpen for document: {} (published diagnostics)", uri)).await;

    Ok(())
  }

  async fn document_highlight(
    &self,
    params: lsp::DocumentHighlightParams,
  ) -> Result<Option<Vec<lsp::DocumentHighlight>>, jsonrpc::Error> {
    let uri = params.text_document_position_params.text_document.uri;

    let position = params.text_document_position_params.position;

    Ok(self.documents.get(&uri).and_then(|document| {
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

  async fn execute_command(
    &self,
    params: lsp::ExecuteCommandParams,
  ) -> Result<Option<serde_json::Value>, jsonrpc::Error> {
    match Command::try_from(params.command.as_str()) {
      Ok(Command::RunRecipe) => {
        let recipe_name = params
          .arguments
          .first()
          .and_then(|recipe_name| recipe_name.as_str());

        let uri = params
          .arguments
          .get(1)
          .and_then(|arguments| arguments.as_str())
          .and_then(|arguments| lsp::Url::parse(arguments).ok());

        let parameters = params.arguments.get(2).and_then(|parameters| {
          serde_json::from_value::<Vec<ParameterJson>>(parameters.clone()).ok()
        });

        if let (Some(recipe_name), Some(uri), Some(parameters)) =
          (recipe_name, uri, parameters)
        {
          let path = uri
            .to_file_path()
            .ok()
            .and_then(|path| path.parent().map(|p| p.to_path_buf()))
            .unwrap_or(PathBuf::new());

          let recipe_arguments = Vec::new();

          if !parameters.is_empty() {
            self.client.show_message(
              lsp::MessageType::WARNING,
              "Running a recipe code action with parameters is not yet supported."
            )
            .await;

            return Ok(None);
          }

          self.run_recipe(recipe_name, recipe_arguments, path).await;
        }
      }
      Err(error) => {
        self
          .client
          .show_message(lsp::MessageType::ERROR, error)
          .await;
      }
    }

    Ok(None)
  }

  async fn folding_range(
    &self,
    params: lsp::FoldingRangeParams,
  ) -> Result<Option<Vec<lsp::FoldingRange>>, jsonrpc::Error> {
    let uri = &params.text_document.uri;

    if let Some(document) = self.documents.get(uri) {
      let recipes = document.get_recipes();

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
    let uri = &params.text_document.uri;

    if let Some(document) = self.documents.get(uri) {
      let content = document.content.to_string();

      match self.format_document(&content).await {
        Ok(formatted) => {
          if formatted != content {
            return Ok(Some(vec![lsp::TextEdit {
              range: lsp::Range {
                start: lsp::Position::new(0, 0),
                end: document
                  .content
                  .byte_to_lsp_position(document.content.len_bytes()),
              },
              new_text: formatted,
            }]));
          }

          return Ok(Some(vec![]));
        }
        Err(error) => {
          self
            .client
            .show_message(
              lsp::MessageType::ERROR,
              format!("Failed to format document: {error}"),
            )
            .await;
        }
      }
    }

    Ok(None)
  }

  async fn format_document(&self, content: &str) -> Result<String> {
    let tempdir = tempdir()?;

    let file = tempdir.path().join("justfile");

    fs::write(&file, content.as_bytes())?;

    let mut command = tokio::process::Command::new("just");

    command.arg("--fmt").arg("--unstable").arg("--quiet");

    command.current_dir(tempdir.path());

    let output = command.output().await?;

    if !output.status.success() {
      bail!(
        "just formatting failed: {}",
        String::from_utf8_lossy(&output.stderr)
      );
    }

    Ok(fs::read_to_string(&file)?)
  }

  async fn goto_definition(
    &self,
    params: lsp::GotoDefinitionParams,
  ) -> Result<Option<lsp::GotoDefinitionResponse>, jsonrpc::Error> {
    let uri = params.text_document_position_params.text_document.uri;

    let position = params.text_document_position_params.position;

    Ok(self.documents.get(&uri).and_then(|document| {
      document
        .node_at_position(position)
        .filter(|node| node.kind() == "identifier")
        .and_then(|identifier| {
          let resolver = Resolver::new(document);

          resolver
            .resolve_identifier_definition(&identifier)
            .map(lsp::GotoDefinitionResponse::Scalar)
        })
    }))
  }

  async fn hover(
    &self,
    params: lsp::HoverParams,
  ) -> Result<Option<lsp::Hover>, jsonrpc::Error> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;
    
    self.debug_logger.log(&format!("RECEIVED: hover request for {}:{}:{}", uri, position.line, position.character)).await;

    let result = self.documents.get(&uri).and_then(|document| {
      let resolver = Resolver::new(document);

      document
        .node_at_position(position)
        .filter(|node| node.kind() == "identifier")
        .and_then(|identifier| resolver.resolve_identifier_hover(&identifier))
    });
    
    self.debug_logger.log(&format!("SENDING: hover response: {}", 
      if result.is_some() { "found hover info" } else { "no hover info" })).await;
    
    Ok(result)
  }

  async fn initialize(
    &self,
    params: lsp::InitializeParams,
  ) -> Result<lsp::InitializeResult, jsonrpc::Error> {
    self.debug_logger.log(&format!("RECEIVED: initialize request with params: {}", 
      serde_json::to_string(&params).unwrap_or_else(|_| "failed to serialize".to_string()))).await;
    
    log::info!("Starting just language server...");

    let result = lsp::InitializeResult {
      capabilities: Server::capabilities(),
      server_info: Some(lsp::ServerInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
      }),
    };
    
    self.debug_logger.log(&format!("SENDING: initialize response: {}", 
      serde_json::to_string(&result).unwrap_or_else(|_| "failed to serialize".to_string()))).await;
    
    Ok(result)
  }

  async fn initialized(&mut self, _: lsp::InitializedParams) {
    self
      .client
      .log_message(
        lsp::MessageType::INFO,
        &format!("{} initialized", env!("CARGO_PKG_NAME")),
      )
      .await;

    self.initialized = true;
  }

  async fn publish_diagnostics(&self, uri: &lsp::Url) {
    if !self.initialized {
      return;
    }

    if let Some(document) = self.documents.get(uri) {
      let analyzer = Analyzer::new(document);

      self
        .client
        .publish_diagnostics(
          uri.clone(),
          analyzer.analyze(),
          Some(document.version),
        )
        .await;
    }
  }

  async fn references(
    &self,
    params: lsp::ReferenceParams,
  ) -> Result<Option<Vec<lsp::Location>>, jsonrpc::Error> {
    let uri = params.text_document_position.text_document.uri;

    let position = params.text_document_position.position;

    Ok(self.documents.get(&uri).and_then(|document| {
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

    Ok(self.documents.get(&uri).and_then(|document| {
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

  async fn run_recipe(
    &self,
    recipe_name: &str,
    recipe_arguments: Vec<String>,
    directory: PathBuf,
  ) {
    let document_uri = lsp::Url::parse(&format!(
      "just-recipe:/{}/{}",
      directory.display(),
      recipe_name
    ))
    .unwrap_or_else(|_| lsp::Url::parse("just-recipe:/output").unwrap());

    let mut command = tokio::process::Command::new("just");

    command.arg(recipe_name);

    for argument in recipe_arguments {
      command.arg(argument);
    }

    command
      .current_dir(directory.clone())
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped());

    let client = self.client.clone();

    client
      .show_document(lsp::ShowDocumentParams {
        uri: document_uri.clone(),
        external: Some(false),
        take_focus: Some(true),
        selection: None,
      })
      .await
      .ok();

    let changes = HashMap::from([(
      document_uri.clone(),
      vec![lsp::TextEdit {
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0,
          },
          end: lsp::Position {
            line: u32::MAX,
            character: 0,
          },
        },
        new_text: String::new(),
      }],
    )]);

    client
      .apply_edit(lsp::WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
      })
      .await
      .ok();

    let recipe_name = recipe_name.to_string();

    tokio::spawn(async move {
      match command.spawn() {
        Ok(mut child) => {
          let stdout_lines = LinesStream::new(
            tokio::io::BufReader::new(
              child.stdout.take().expect("Failed to capture stdout"),
            )
            .lines(),
          );

          let stderr_lines = LinesStream::new(
            tokio::io::BufReader::new(
              child.stderr.take().expect("Failed to capture stderr"),
            )
            .lines(),
          );

          let mut merged_stream = StreamExt::merge(stdout_lines, stderr_lines);

          let mut buffer = String::new();
          let mut current_line = 0;
          let mut last_update = std::time::Instant::now();

          while let Some(line_result) = merged_stream.next().await {
            match line_result {
              Ok(line) => {
                buffer.push_str(&line);

                buffer.push('\n');

                let now = std::time::Instant::now();

                if (now.duration_since(last_update).as_millis() > 50
                  || buffer.len() > 1024)
                  && !buffer.is_empty()
                {
                  let changes = HashMap::from([(
                    document_uri.clone(),
                    vec![lsp::TextEdit {
                      range: lsp::Range {
                        start: lsp::Position {
                          line: current_line,
                          character: 0,
                        },
                        end: lsp::Position {
                          line: current_line,
                          character: 0,
                        },
                      },
                      new_text: buffer.trim().into(),
                    }],
                  )]);

                  client
                    .apply_edit(lsp::WorkspaceEdit {
                      changes: Some(changes),
                      ..Default::default()
                    })
                    .await
                    .ok();

                  let newlines = buffer.matches('\n').count() as u32;

                  current_line += newlines;
                  buffer.clear();
                  last_update = now;
                }
              }
              Err(error) => {
                buffer.push_str(&format!("Error reading output: {error}\n"));
              }
            }
          }

          if !buffer.is_empty() {
            let changes = HashMap::from([(
              document_uri.clone(),
              vec![lsp::TextEdit {
                range: lsp::Range {
                  start: lsp::Position {
                    line: current_line,
                    character: 0,
                  },
                  end: lsp::Position {
                    line: current_line,
                    character: 0,
                  },
                },
                new_text: buffer.trim().into(),
              }],
            )]);

            client
              .apply_edit(lsp::WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
              })
              .await
              .ok();
          }

          match child.wait().await {
            Ok(status) => {
              if !status.success() {
                client
                  .show_message(
                    lsp::MessageType::WARNING,
                    format!("Recipe '{recipe_name}' completed with non-zero exit code: {status}"),
                  )
                  .await;
              }
            }
            Err(error) => {
              client
                .show_message(
                  lsp::MessageType::ERROR,
                  format!("Error waiting for recipe '{recipe_name}': {error}"),
                )
                .await;
            }
          }
        }
        Err(error) => {
          client
            .show_message(
              lsp::MessageType::ERROR,
              format!("Failed to run recipe '{recipe_name}': {error}"),
            )
            .await;
        }
      }
    });
  }

  async fn shutdown(&mut self) -> Result<(), jsonrpc::Error> {
    self.debug_logger.log("RECEIVED: shutdown request").await;
    self.shutdown_received = true;
    Ok(())
  }

  
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::indoc,
    pretty_assertions::assert_eq,
    serde_json::{json, Value},
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
    fn new() -> Result<Self> {
      let (service, _) = LspService::new(Server::new);

      Ok(Self {
        requests: Vec::new(),
        responses: Vec::new(),
        service: Spawn::new(service),
      })
    }

    fn request<T: IntoValue>(mut self, request: T) -> Self {
      self.requests.push(request.into_value());
      self
    }

    fn response<T: IntoValue>(mut self, response: T) -> Self {
      self.responses.push(Some(response.into_value()));
      self
    }

    fn notification<T: IntoValue>(mut self, notification: T) -> Self {
      self.requests.push(notification.into_value());
      self.responses.push(None);
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
    uri: &'a str,
    text: &'a str,
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
    uri: &'a str,
    version: i32,
    changes: Vec<lsp::TextDocumentContentChangeEvent>,
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
  struct GotoDefinitionRequest<'a> {
    id: i64,
    uri: &'a str,
    line: u32,
    character: u32,
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
    id: i64,
    uri: &'a str,
    start_line: u32,
    start_char: u32,
    end_line: u32,
    end_char: u32,
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
    uri: &'a str,
    start_line: u32,
    start_char: u32,
    end_line: u32,
    end_char: u32,
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
    id: i64,
    uri: &'a str,
    line: u32,
    character: u32,
    include_declaration: bool,
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
      self
        .into_iter()
        .map(|location| location.into_value())
        .collect()
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
    start_line: u32,
    start_char: u32,
    end_line: u32,
    end_char: u32,
    new_text: &'a str,
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
    id: i64,
    uri: &'a str,
    line: u32,
    character: u32,
    new_name: &'a str,
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
    id: i64,
    uri: &'a str,
    edits: Vec<Rename<'a>>,
  }

  impl IntoValue for Vec<Rename<'_>> {
    fn into_value(self) -> Value {
      self.into_iter().map(|edit| edit.into_value()).collect()
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
  struct HoverRequest<'a> {
    id: i64,
    uri: &'a str,
    line: u32,
    character: u32,
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
    id: i64,
    content: &'a str,
    kind: &'a str,
    start_line: u32,
    start_char: u32,
    end_line: u32,
    end_char: u32,
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
    id: i64,
    uri: &'a str,
    line: u32,
    character: u32,
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
    id: i64,
    highlights: Vec<Highlight<'a>>,
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
    start_line: u32,
    start_char: u32,
    end_line: u32,
    end_char: u32,
    kind: &'a str,
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
          "text" => 1,
          "read" => 2,
          "write" => 3,
          _ => 1
        }
      })
    }
  }

  impl IntoValue for Vec<Highlight<'_>> {
    fn into_value(self) -> Value {
      self
        .into_iter()
        .map(|highlight| highlight.into_value())
        .collect()
    }
  }

  #[derive(Debug)]
  struct FoldingRange<'a> {
    start_line: u32,
    end_line: u32,
    kind: &'a str,
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
      self.into_iter().map(|range| range.into_value()).collect()
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
    uri: &'static str,
    range: lsp::Range,
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
    id: i64,
    actions: Vec<CodeAction>,
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
    title: &'static str,
    kind: &'static str,
    command: Command,
    arguments: Vec<ParameterJson>,
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
      self.into_iter().map(|a| a.into_value()).collect()
    }
  }

  #[tokio::test]
  async fn initialize() -> Result {
    Test::new()?
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .run()
      .await
  }

  #[tokio::test]
  async fn initialize_once() -> Result {
    Test::new()?
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
  async fn shutdown() -> Result {
    Test::new()?
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

  #[tokio::test]
  async fn did_change_updates_document() -> Result {
    Test::new()?
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
          range: Some(lsp::Range {
            start: lsp::Position {
              line: 1,
              character: 7,
            },
            end: lsp::Position {
              line: 1,
              character: 13,
            },
          }),
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
  async fn goto_recipe_definition_from_dependency() -> Result {
    Test::new()?
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
  async fn recipe_references() -> Result {
    Test::new()?
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
    Test::new()?
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
  async fn hover_builtin_function() -> Result {
    Test::new()?
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
        content: "Instruction set architecture\n```\narch() -> string\n```\n**Examples:**\n```\narch() => \"x86_64\"\n```",
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
  async fn hover_recipe() -> Result {
    Test::new()?
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
  async fn hover_constant() -> Result {
    Test::new()?
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
        content: "Lowercase hexadecimal digit string\n\"0123456789abcdef\"",
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
  async fn hover_same_named_recipes_and_functions() -> Result {
    Test::new()?
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
        content: "Instruction set architecture\n```\narch() -> string\n```\n**Examples:**\n```\narch() => \"x86_64\"\n```",
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
  async fn hover_attribute() -> Result {
    Test::new()?
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
        content: "**Attribute**: [no-cd]\nDon't change directory before executing recipe.\n**Introduced in**: 1.9.0\n**Target(s)**: recipe",
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
  async fn hover_setting() -> Result {
    Test::new()?
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
        content: "**Setting**: export\nExport all variables as environment variables.\n**Type**: boolean\n**Default**: false",
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
    Test::new()?
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
  async fn hover_recipe_parameter_in_interpolation() -> Result {
    Test::new()?
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
  async fn hover_prioritize_recipe_parameter_over_variable_in_interpolation(
  ) -> Result {
    Test::new()?
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
  async fn hover_local_parameter() -> Result {
    Test::new()?
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
  async fn document_highlight() -> Result {
    Test::new()?
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
  async fn folding_range() -> Result {
    Test::new()?
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

  #[tokio::test]
  async fn code_action_empty_document() -> Result {
    Test::new()?
      .request(InitializeRequest { id: 1 })
      .response(InitializeResponse { id: 1 })
      .notification(DidOpenNotification {
        uri: "file:///empty.just",
        text: "",
      })
      .request(CodeActionRequest {
        id: 2,
        uri: "file:///empty.just",
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0,
          },
          end: lsp::Position {
            line: 0,
            character: 0,
          },
        },
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
    Test::new()?
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
        range: lsp::Range {
          start: lsp::Position {
            line: 0,
            character: 0,
          },
          end: lsp::Position {
            line: 0,
            character: 0,
          },
        },
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
}
