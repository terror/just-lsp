use {
  crate::{capabilities::server_capabilities, language_server::LanguageServer},
  lspower::{
    jsonrpc,
    lsp::{
      DidChangeTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
      HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams, MessageType,
      SaveOptions, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
      TextDocumentSyncKind, TextDocumentSyncOptions,
    },
    Client, LspService, Server,
  },
  std::{env, process},
};

mod capabilities;
mod language_server;

pub(crate) type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

async fn run() -> Result {
  let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

  let (service, messages) = LspService::new(|client| LanguageServer::new(client));

  Server::new(stdin, stdout)
    .interleave(messages)
    .serve(service)
    .await;

  Ok(())
}

#[tokio::main]
async fn main() {
  env_logger::init();
  if let Err(error) = run().await {
    println!("error: {error}");
    process::exit(1);
  }
}
