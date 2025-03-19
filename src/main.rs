use {
  crate::{
    analyzer::Analyzer,
    attribute::{Attribute, AttributeTarget},
    constant::Constant,
    document::Document,
    function::Function,
    node_ext::NodeExt,
    point_ext::PointExt,
    position_ext::PositionExt,
    recipe::Recipe,
    server::Server,
    setting::{Setting, SettingKind},
  },
  lsp_text::RopeExt,
  lspower::{jsonrpc, lsp, Client, LanguageServer, LspService},
  ropey::Rope,
  std::{
    collections::{BTreeMap, HashSet},
    env, process,
    sync::Arc,
  },
  tree_sitter::{Language, Node, Parser, Point, Tree, TreeCursor},
};

mod analyzer;
mod attribute;
mod constant;
mod constants;
mod document;
mod function;
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
