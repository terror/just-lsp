use super::*;

#[derive(Debug)]
pub struct Server(Arc<tokio::sync::Mutex<Inner>>);

impl Server {
  pub fn new(client: Client) -> Self {
    Self(Arc::new(tokio::sync::Mutex::new(Inner::new(client))))
  }

  pub async fn run() -> Result {
    let (service, messages) = LspService::new(Server::new);

    lspower::Server::new(tokio::io::stdin(), tokio::io::stdout())
      .interleave(messages)
      .serve(service)
      .await;

    Ok(())
  }

  pub fn capabilities() -> lsp::ServerCapabilities {
    lsp::ServerCapabilities {
      completion_provider: Some(lsp::CompletionOptions {
        ..Default::default()
      }),
      definition_provider: Some(lsp::OneOf::Left(true)),
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

#[lspower::async_trait]
impl LanguageServer for Server {
  async fn completion(
    &self,
    params: lsp::CompletionParams,
  ) -> Result<Option<lsp::CompletionResponse>, jsonrpc::Error> {
    self.0.lock().await.completion(params).await
  }

  async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
    if let Err(error) = self.0.lock().await.did_change(params).await {
      log::debug!("error: {error}");
    }
  }

  async fn did_close(&self, params: lsp::DidCloseTextDocumentParams) {
    self.0.lock().await.did_close(params).await
  }

  async fn did_open(&self, params: lsp::DidOpenTextDocumentParams) {
    if let Err(error) = self.0.lock().await.did_open(params).await {
      log::debug!("error: {error}");
    }
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
}

impl Inner {
  fn new(client: Client) -> Self {
    Self {
      client,
      documents: BTreeMap::new(),
      initialized: false,
    }
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
    let uri = params.text_document.uri.clone();

    self.documents.insert(
      params.text_document.uri.to_owned(),
      Document::try_from(params)?,
    );

    self.publish_diagnostics(&uri).await;

    Ok(())
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
        .and_then(|node| {
          node
            .parent()
            .filter(|parent| {
              parent.kind() == "dependency" || parent.kind() == "alias"
            })
            .map(|_| node)
        })
        .and_then(|node| {
          let recipe_name = document.get_node_text(&node);

          document.find_recipe(&recipe_name).map(|recipe| {
            lsp::GotoDefinitionResponse::Scalar(lsp::Location {
              uri: uri.clone(),
              range: recipe.range,
            })
          })
        })
    }))
  }

  async fn hover(
    &self,
    params: lsp::HoverParams,
  ) -> Result<Option<lsp::Hover>, jsonrpc::Error> {
    let uri = params.text_document_position_params.text_document.uri;

    let position = params.text_document_position_params.position;

    Ok(self.documents.get(&uri).and_then(|document| {
      document
        .node_at_position(position)
        .filter(|node| node.kind() == "identifier")
        .and_then(|node| {
          let text = document.get_node_text(&node);

          let parent_kind = node.parent().map(|p| p.kind());

          if let Some(recipe) = document.find_recipe(&text) {
            if parent_kind.is_some_and(|kind| {
              ["alias", "dependency", "recipe_header"].contains(&kind)
            }) {
              return Some(lsp::Hover {
                contents: lsp::HoverContents::Markup(lsp::MarkupContent {
                  kind: lsp::MarkupKind::PlainText,
                  value: recipe.content,
                }),
                range: Some(node.get_range()),
              });
            }
          }

          if parent_kind.is_some_and(|kind| kind == "value") {
            let recipes = document.get_recipes();

            for recipe in recipes {
              for parameter in recipe.parameters {
                if parameter.name == text {
                  return Some(lsp::Hover {
                    contents: lsp::HoverContents::Markup(lsp::MarkupContent {
                      kind: lsp::MarkupKind::PlainText,
                      value: parameter.content,
                    }),
                    range: Some(node.get_range()),
                  });
                }
              }
            }

            let variables = document.get_variables();

            for variable in variables {
              if variable.name == text {
                return Some(lsp::Hover {
                  contents: lsp::HoverContents::Markup(lsp::MarkupContent {
                    kind: lsp::MarkupKind::PlainText,
                    value: variable.content,
                  }),
                  range: Some(node.get_range()),
                });
              }
            }
          }

          for builtin in builtins::BUILTINS {
            match builtin {
              Builtin::Attribute { name, .. } if text == name => {
                if parent_kind.is_some_and(|kind| kind == "attribute") {
                  return Some(lsp::Hover {
                    contents: lsp::HoverContents::Markup(
                      builtin.documentation(),
                    ),
                    range: Some(node.get_range()),
                  });
                }
              }
              Builtin::Constant { name, .. }
              | Builtin::Function { name, .. }
              | Builtin::Setting { name, .. }
                if text == name =>
              {
                return Some(lsp::Hover {
                  contents: lsp::HoverContents::Markup(builtin.documentation()),
                  range: Some(node.get_range()),
                });
              }
              _ => {}
            }
          }

          None
        })
    }))
  }

  async fn initialize(
    &self,
    _params: lsp::InitializeParams,
  ) -> Result<lsp::InitializeResult, jsonrpc::Error> {
    log::info!("Starting just language server...");

    Ok(lsp::InitializeResult {
      capabilities: Server::capabilities(),
      server_info: Some(lsp::ServerInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
      }),
    })
  }

  async fn initialized(&mut self, _: lsp::InitializedParams) {
    self
      .client
      .show_message(
        lsp::MessageType::INFO,
        &format!("{} initialized", env!("CARGO_PKG_NAME")),
      )
      .await;

    self.initialized = true;
  }

  async fn publish_diagnostics(&self, uri: &lsp::Url) {
    if self.initialized {
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
  }

  async fn references(
    &self,
    params: lsp::ReferenceParams,
  ) -> Result<Option<Vec<lsp::Location>>, jsonrpc::Error> {
    let uri = params.text_document_position.text_document.uri;

    let position = params.text_document_position.position;

    Ok(self.documents.get(&uri).and_then(|document| {
      document
        .node_at_position(position)
        .filter(|node| node.kind() == "identifier")
        .map(|identifier| {
          document.find_references(&document.get_node_text(&identifier))
        })
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
        .map(|node| {
          let old_name = document.get_node_text(&node);

          let references = document.find_references(&old_name);

          let text_edits: Vec<lsp::TextEdit> = references
            .iter()
            .map(|location| lsp::TextEdit {
              range: location.range,
              new_text: new_name.clone(),
            })
            .collect();

          let mut changes = std::collections::HashMap::new();

          changes.insert(uri.clone(), text_edits);

          lsp::WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
          }
        })
    }))
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
    lspower::{LspService, MessageStream},
    pretty_assertions::assert_eq,
    serde_json::{json, Value},
    std::env,
    tower_test::mock::Spawn,
  };

  #[derive(Debug)]
  struct Test {
    _messages: MessageStream,
    requests: Vec<Value>,
    responses: Vec<Option<Value>>,
    service: Spawn<LspService>,
  }

  impl Test {
    fn new() -> Result<Self> {
      let (service, messages) = LspService::new(Server::new);

      Ok(Self {
        _messages: messages,
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
        content: "**Setting**: export\nExport all variables as environment variables.\n**Type**: Boolean\n**Default**: false",
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
}
