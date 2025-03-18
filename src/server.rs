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
      definition_provider: Some(lsp::OneOf::Left(true)),
      references_provider: Some(lsp::OneOf::Left(true)),
      ..Default::default()
    }
  }
}

#[lspower::async_trait]
impl LanguageServer for Server {
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
      .show(Message {
        content: &format!("{} initialized", env!("CARGO_PKG_NAME")),
        kind: lsp::MessageType::INFO,
      })
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
            None,
          )
          .await;
      }
    }
  }

  async fn show(&self, message: Message<'_>) {
    self
      .client
      .show_message(message.kind, message.content)
      .await;
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
        .and_then(|identifier| {
          Some(document.find_references(&document.get_node_text(&identifier)))
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
    responses: Vec<Value>,
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
      self.responses.push(response);
      self
    }

    async fn run(mut self) -> Result {
      for (request, expected_response) in
        self.requests.iter().zip(self.responses.iter())
      {
        assert_eq!(
          *expected_response,
          self
            .service
            .call(serde_json::from_value(request.clone())?)
            .await?
            .map(|v| serde_json::to_value(v).unwrap())
            .unwrap()
        );
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
}
