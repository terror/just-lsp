use {
  crate::{
    alias::Alias,
    analyzer::Analyzer,
    builtin::{AttributeTarget, Builtin},
    command::Command,
    document::Document,
    node_ext::NodeExt,
    point_ext::PointExt,
    position_ext::PositionExt,
    recipe::{Dependency, Parameter, ParameterJson, ParameterKind, Recipe},
    resolver::Resolver,
    rope_ext::RopeExt,
    server::Server,
    setting::{Setting, SettingKind},
    text_node::TextNode,
    variable::Variable,
  },
  anyhow::{anyhow, Error},
  ropey::Rope,
  serde::{Deserialize, Serialize},
  std::{
    collections::{BTreeMap, HashMap, HashSet},
    env,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    path::PathBuf,
    process,
    sync::Arc,
  },
  tokio::io::AsyncBufReadExt,
  tokio_stream::{wrappers::LinesStream, StreamExt},
  tower_lsp::{jsonrpc, lsp_types as lsp, Client, LanguageServer, LspService},
  tree_sitter::{Language, Node, Parser, Point, Tree, TreeCursor},
};

mod alias;
mod analyzer;
mod builtin;
mod builtins;
mod command;
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

type Result<T = (), E = Error> = std::result::Result<T, E>;

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
