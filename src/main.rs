use {
  crate::{
    alias::Alias,
    analyzer::Analyzer,
    builtin::{AttributeTarget, Builtin},
    document::Document,
    node_ext::NodeExt,
    point_ext::PointExt,
    position_ext::PositionExt,
    recipe::{Parameter, ParameterKind, Recipe},
    server::Server,
    setting::{Setting, SettingKind},
  },
  lsp_text::RopeExt,
  lspower::{jsonrpc, lsp, Client, LanguageServer, LspService},
  ropey::Rope,
  std::{
    collections::{BTreeMap, HashSet},
    env,
    fmt::{self, Display, Formatter},
    process,
    sync::Arc,
  },
  tree_sitter::{Language, Node, Parser, Point, Tree, TreeCursor},
};

mod alias;
mod analyzer;
mod builtin;
mod builtins;
mod document;
mod node_ext;
mod point_ext;
mod position_ext;
mod recipe;
mod server;
mod setting;

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
