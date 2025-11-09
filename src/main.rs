use {
  alias::Alias,
  analyzer::Analyzer,
  anyhow::{anyhow, bail, Error},
  arguments::Arguments,
  ariadne::{sources, Color, Label, Report, ReportKind},
  attribute::Attribute,
  builtin::{AttributeTarget, Builtin},
  clap::Parser as Clap,
  command::Command,
  count::Count,
  document::Document,
  env_logger::Env,
  node_ext::NodeExt,
  once_cell::sync::OnceCell,
  os_group::OsGroup,
  point_ext::PointExt,
  position_ext::PositionExt,
  recipe::{Dependency, Parameter, ParameterJson, ParameterKind, Recipe},
  resolver::Resolver,
  rope_ext::RopeExt,
  ropey::Rope,
  rule::{RuleContext, RULES},
  serde::{Deserialize, Serialize},
  server::Server,
  setting::{Setting, SettingKind},
  std::{
    backtrace::BacktraceStatus,
    collections::{BTreeMap, HashMap, HashSet},
    env,
    fmt::{self, Debug, Display, Formatter},
    fs,
    path::PathBuf,
    process,
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
  },
  subcommand::Subcommand,
  tempfile::tempdir,
  text_node::TextNode,
  tokio::{io::AsyncBufReadExt, sync::RwLock},
  tokio_stream::{wrappers::LinesStream, StreamExt},
  tower_lsp::{jsonrpc, lsp_types as lsp, Client, LanguageServer, LspService},
  tree_sitter::{Language, Node, Parser, Point, Tree, TreeCursor},
  variable::Variable,
};

mod alias;
mod analyzer;
mod arguments;
mod attribute;
mod builtin;
mod builtins;
mod command;
mod count;
mod document;
mod node_ext;
mod os_group;
mod point_ext;
mod position_ext;
mod recipe;
mod resolver;
mod rope_ext;
mod rule;
mod server;
mod setting;
mod subcommand;
mod text_node;
mod variable;

type Result<T = (), E = Error> = std::result::Result<T, E>;

extern "C" {
  pub(crate) fn tree_sitter_just() -> Language;
}

#[tokio::main]
async fn main() {
  let env = Env::default().default_filter_or("info");

  env_logger::Builder::from_env(env).init();

  if let Err(error) = Arguments::parse().run().await {
    eprintln!("error: {error}");

    for (i, error) in error.chain().skip(1).enumerate() {
      if i == 0 {
        eprintln!();
        eprintln!("because:");
      }

      eprintln!("- {error}");
    }

    let backtrace = error.backtrace();

    if backtrace.status() == BacktraceStatus::Captured {
      eprintln!("backtrace:");
      eprintln!("{backtrace}");
    }

    process::exit(1);
  }
}
