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

      let recipe_names = document.get_recipe_names();

      for name in recipe_names {
        completion_items.push(lsp::CompletionItem {
          label: name.clone(),
          kind: Some(lsp::CompletionItemKind::FUNCTION),
          detail: Some("Recipe".to_string()),
          insert_text: Some(name),
          insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
          ..Default::default()
        });
      }

      for (name, signature, description) in builtin_functions() {
        let insert_text = create_function_snippet(&name);

        let documentation = get_function_documentation(&name, &description);

        completion_items.push(lsp::CompletionItem {
          label: name.clone(),
          kind: Some(lsp::CompletionItemKind::FUNCTION),
          detail: Some(signature.clone()),
          documentation: Some(lsp::Documentation::MarkupContent(
            lsp::MarkupContent {
              kind: lsp::MarkupKind::Markdown,
              value: documentation,
            },
          )),
          insert_text: Some(insert_text),
          insert_text_format: Some(lsp::InsertTextFormat::SNIPPET),
          sort_text: Some(format!("z{}", name)),
          ..Default::default()
        });
      }

      for (name, value) in builtin_constants() {
        completion_items.push(lsp::CompletionItem {
          label: name.clone(),
          kind: Some(lsp::CompletionItemKind::CONSTANT),
          detail: Some("Constant".into()),
          documentation: Some(lsp::Documentation::MarkupContent(
            lsp::MarkupContent {
              kind: lsp::MarkupKind::Markdown,
              value,
            },
          )),
          insert_text: Some(name.clone()),
          insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
          sort_text: Some(format!("z{}", name)),
          ..Default::default()
        });
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

          document
            .find_recipe_by_name(&recipe_name)
            .map(|recipe_node| {
              lsp::GotoDefinitionResponse::Scalar(lsp::Location {
                uri: uri.clone(),
                range: document.node_to_range(&recipe_node),
              })
            })
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
        self
          .client
          .publish_diagnostics(
            uri.clone(),
            document.collect_diagnostics(),
            Some(document.version()),
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
            document_changes: None,
            change_annotations: None,
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

    fn request(mut self, request: Value) -> Self {
      self.requests.push(request);
      self
    }

    fn response(mut self, response: Value) -> Self {
      self.responses.push(Some(response));
      self
    }

    fn notification(mut self, notification: Value) -> Self {
      self.requests.push(notification);
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

  #[tokio::test]
  async fn initialize() -> Result {
    Test::new()?
      .request(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
          "capabilities": {}
        },
      }))
      .response(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
          "serverInfo": {
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION")
          },
          "capabilities": Server::capabilities()
        },
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn initialize_once() -> Result {
    Test::new()?
      .request(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
          "capabilities": {}
        },
      }))
      .response(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
          "serverInfo": {
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION")
          },
          "capabilities": Server::capabilities()
        }
      }))
      .request(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
          "capabilities": {}
        }
      }))
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
  async fn goto_definition() -> Result {
    Test::new()?
      .request(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
          "capabilities": {}
        },
      }))
      .response(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
          "serverInfo": {
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION")
          },
          "capabilities": Server::capabilities()
        },
      }))
      .notification(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
          "textDocument": {
            "uri": "file:///test.just",
            "languageId": "just",
            "version": 1,
            "text": indoc! {
              "
              foo:
                echo \"foo\"

              bar: foo
                echo \"bar\"
              "
            }
          }
        }
      }))
      .request(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/definition",
        "params": {
          "textDocument": {
            "uri": "file:///test.just"
          },
          "position": {
            "line": 3,
            "character": 5
          }
        }
      }))
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": {
          "uri": "file:///test.just",
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 3,
              "character": 0
            }
          }
        }
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn references() -> Result {
    Test::new()?
      .request(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
          "capabilities": {}
        },
      }))
      .response(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
          "serverInfo": {
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION")
          },
          "capabilities": Server::capabilities()
        },
      }))
      .notification(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
          "textDocument": {
            "uri": "file:///test.just",
            "languageId": "just",
            "version": 1,
            "text": indoc! {
              "
              foo:
                echo \"foo\"

              bar: foo
                echo \"bar\"

              alias baz := foo
              "
            }
          }
        }
      }))
      .request(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/references",
        "params": {
          "textDocument": {
            "uri": "file:///test.just"
          },
          "position": {
            "line": 0,
            "character": 1
          },
          "context": {
            "includeDeclaration": true
          }
        }
      }))
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": [
          {
            "uri": "file:///test.just",
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 3
              }
            }
          },
          {
            "uri": "file:///test.just",
            "range": {
              "start": {
                "line": 3,
                "character": 5
              },
              "end": {
                "line": 3,
                "character": 8
              }
            }
          },
          {
            "uri": "file:///test.just",
            "range": {
              "start": {
                "line": 6,
                "character": 13
              },
              "end": {
                "line": 6,
                "character": 16
              }
            }
          }
        ]
      }))
      .run()
      .await
  }

  #[tokio::test]
  async fn rename() -> Result {
    Test::new()?
      .request(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
          "capabilities": {}
        },
      }))
      .response(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
          "serverInfo": {
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION")
          },
          "capabilities": Server::capabilities()
        },
      }))
      .notification(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
          "textDocument": {
            "uri": "file:///test.just",
            "languageId": "just",
            "version": 1,
            "text": indoc! {
              "
              foo:
                echo \"foo\"

              bar: foo
                echo \"bar\"

              alias baz := foo
              "
            }
          }
        }
      }))
      .request(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/rename",
        "params": {
          "textDocument": {
            "uri": "file:///test.just"
          },
          "position": {
            "line": 0,
            "character": 1
          },
          "newName": "renamed"
        }
      }))
      .response(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": {
          "changes": {
            "file:///test.just": [
              {
                "range": {
                  "start": {
                    "line": 0,
                    "character": 0
                  },
                  "end": {
                    "line": 0,
                    "character": 3
                  }
                },
                "newText": "renamed"
              },
              {
                "range": {
                  "start": {
                    "line": 3,
                    "character": 5
                  },
                  "end": {
                    "line": 3,
                    "character": 8
                  }
                },
                "newText": "renamed"
              },
              {
                "range": {
                  "start": {
                    "line": 6,
                    "character": 13
                  },
                  "end": {
                    "line": 6,
                    "character": 16
                  }
                },
                "newText": "renamed"
              }
            ]
          }
        }
      }))
      .run()
      .await
  }
}
