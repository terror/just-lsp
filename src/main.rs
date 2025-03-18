use {
  crate::{document::Document, message::Message, server::Server},
  lsp_text::RopeExt,
  lspower::{jsonrpc, lsp, Client, LanguageServer, LspService},
  ropey::Rope,
  std::{collections::BTreeMap, env, process, sync::Arc},
  tree_sitter::{Language, Node, Parser, Point, Tree},
};

mod document;
mod message;
mod server;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

extern "C" {
  pub(crate) fn tree_sitter_just() -> Language;
}

#[tokio::main]
async fn main() {
  env_logger::init();

  if let Err(error) = Server::run().await {
    println!("error: {error}");
    process::exit(1);
  }
}
