use crate::common::*;

#[derive(Debug)]
pub(crate) struct Server(Arc<tokio::sync::Mutex<Inner>>);

impl Server {
  pub(crate) fn new(client: Client) -> Self {
    Self(Arc::new(tokio::sync::Mutex::new(Inner::new(client))))
  }
}

#[derive(Debug)]
pub(crate) struct Inner {
  pub(crate) client: Client,
  documents: Documents,
}

impl Inner {
  fn new(client: Client) -> Self {
    Self {
      client,
      documents: Documents::new(),
    }
  }

  pub(crate) fn capabilities(&self) -> lsp::ServerCapabilities {
    lsp::ServerCapabilities {
      text_document_sync: Some(lsp::TextDocumentSyncCapability::Options(
        lsp::TextDocumentSyncOptions {
          open_close: Some(true),
          change: Some(lsp::TextDocumentSyncKind::INCREMENTAL),
          will_save: None,
          will_save_wait_until: None,
          save: Some(lsp::SaveOptions::default().into()),
        },
      )),
      ..Default::default()
    }
  }

  async fn initialize(
    &self,
    _params: lsp::InitializeParams,
  ) -> Result<lsp::InitializeResult, jsonrpc::Error> {
    log::info!("Starting just language server...");

    Ok(lsp::InitializeResult {
      capabilities: self.capabilities(),
      server_info: Some(lsp::ServerInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
      }),
    })
  }

  async fn initialized(&self, _: lsp::InitializedParams) {
    self
      .client
      .show_message(
        lsp::MessageType::INFO,
        format!("{} initialized", env!("CARGO_PKG_NAME")),
      )
      .await;
  }

  async fn did_open(&mut self, params: lsp::DidOpenTextDocumentParams) {
    self.documents.insert(
      params.text_document.uri.to_owned(),
      Document::from_params(params),
    );
  }

  async fn did_change(&mut self, params: lsp::DidChangeTextDocumentParams) {
    if let Some(document) = self.documents.get_mut(&params.text_document.uri) {
      document.apply_change(params);
    }
  }

  async fn did_close(&mut self, params: lsp::DidCloseTextDocumentParams) {
    let id = &params.text_document.uri;
    if self.documents.contains_key(id) {
      self.documents.remove(id);
    }
  }

  async fn shutdown(&self) -> Result<(), jsonrpc::Error> {
    Ok(())
  }
}

#[lspower::async_trait]
impl LanguageServer for Server {
  async fn initialize(
    &self,
    params: lsp::InitializeParams,
  ) -> Result<lsp::InitializeResult, jsonrpc::Error> {
    self.0.lock().await.initialize(params).await
  }

  async fn initialized(&self, params: lsp::InitializedParams) {
    self.0.lock().await.initialized(params).await
  }

  async fn did_open(&self, params: lsp::DidOpenTextDocumentParams) {
    self.0.lock().await.did_open(params).await
  }

  async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
    self.0.lock().await.did_change(params).await
  }

  async fn did_close(&self, params: lsp::DidCloseTextDocumentParams) {
    self.0.lock().await.did_close(params).await
  }

  async fn shutdown(&self) -> Result<(), jsonrpc::Error> {
    self.0.lock().await.shutdown().await
  }
}