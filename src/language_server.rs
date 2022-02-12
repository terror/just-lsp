use super::*;

#[derive(Debug)]
pub(crate) struct LanguageServer {
  pub(crate) client: Client,
}

impl LanguageServer {
  pub(crate) fn new(client: Client) -> Self {
    Self { client }
  }
}

#[lspower::async_trait]
impl lspower::LanguageServer for LanguageServer {
  async fn initialize(
    &self,
    _params: InitializeParams,
  ) -> Result<InitializeResult, jsonrpc::Error> {
    log::info!("Starting just language server...");

    let capabilities = server_capabilities();

    let server_info = ServerInfo {
      name: env!("CARGO_PKG_NAME").to_string(),
      version: Some(env!("CARGO_PKG_VERSION").to_string()),
    };

    Ok(InitializeResult {
      capabilities,
      server_info: Some(server_info),
    })
  }

  async fn initialized(&self, params: InitializedParams) {
    self
      .client
      .show_message(MessageType::INFO, format!("Initalized: {:?}", params))
      .await;
  }

  async fn did_open(&self, params: DidOpenTextDocumentParams) {
    self
      .client
      .show_message(
        MessageType::INFO,
        format!("Opened a document: {:?}", params),
      )
      .await;
  }

  async fn did_save(&self, params: DidSaveTextDocumentParams) {
    self
      .client
      .show_message(MessageType::INFO, format!("Saved a document: {:?}", params))
      .await;
  }

  async fn did_change(&self, params: DidChangeTextDocumentParams) {
    self
      .client
      .show_message(
        MessageType::INFO,
        format!("Changed a document: {:?}", params),
      )
      .await;
  }

  async fn shutdown(&self) -> Result<(), jsonrpc::Error> {
    Ok(())
  }
}
