use {
  crate::{document::Document, message::Message, server::Server},
  lsp_text::RopeExt,
  lspower::{jsonrpc, lsp, Client, LanguageServer, LspService},
  ropey::Rope,
  std::{collections::BTreeMap, env, process, sync::Arc},
};

mod document;
mod message;
mod server;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[tokio::main]
async fn main() {
  env_logger::init();

  if let Err(error) = Server::run().await {
    println!("error: {error}");
    process::exit(1);
  }
}
