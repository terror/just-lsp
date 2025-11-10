use {
  analyzer::Analyzer,
  anyhow::{anyhow, bail, Error},
  arguments::Arguments,
  ariadne::{sources, Color, Label, Report, ReportKind},
  builtin::{AttributeTarget, Builtin},
  clap::Parser as Clap,
  command::Command,
  count::Count,
  document::Document,
  env_logger::Env,
  just_lsp_rope_ext::RopeExt,
  just_lsp_types::{
    Alias, Attribute, Dependency, FunctionCall, Group, Parameter,
    ParameterJson, ParameterKind, Recipe, Setting, SettingKind, TextNode,
    Variable,
  },
  node_ext::NodeExt,
  once_cell::sync::OnceCell,
  point_ext::PointExt,
  position_ext::PositionExt,
  resolver::Resolver,
  ropey::Rope,
  rule::{RuleContext, RULES},
  server::Server,
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
  tokio::{io::AsyncBufReadExt, sync::RwLock},
  tokio_stream::{wrappers::LinesStream, StreamExt},
  tower_lsp::{jsonrpc, lsp_types as lsp, Client, LanguageServer, LspService},
  tree_sitter::{Language, Node, Parser, Point, Tree, TreeCursor},
};

mod analyzer;
mod arguments;
mod builtin;
mod builtins;
mod command;
mod count;
mod document;
mod node_ext;
mod point_ext;
mod position_ext;
mod resolver;
mod rule;
mod server;
mod subcommand;

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
