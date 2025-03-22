use {
  crate::{
    alias::Alias,
    analyzer::Analyzer,
    builtin::{AttributeTarget, Builtin},
    document::Document,
    node_ext::NodeExt,
    point_ext::PointExt,
    position_ext::PositionExt,
    recipe::{Dependency, Parameter, ParameterKind, Recipe},
    resolver::Resolver,
    rope_ext::RopeExt,
    server::Server,
    setting::{Setting, SettingKind},
    text_node::TextNode,
    variable::Variable,
  },
  lspower::{jsonrpc, lsp, Client, LanguageServer, LspService},
  ropey::Rope,
  std::{
    collections::{BTreeMap, HashMap, HashSet},
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
mod resolver;
mod rope_ext;
mod server;
mod setting;
mod text_node;
mod variable;

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
