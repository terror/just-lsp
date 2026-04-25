use {
  anyhow::{Error, anyhow, bail},
  arguments::Arguments,
  ariadne::{Color, Label, Report, ReportKind, sources},
  clap::{Parser, builder::styling},
  command::Command,
  env_logger::Env,
  just_lsp::*,
  resolver::Resolver,
  ropey::Rope,
  server::Server,
  std::{
    backtrace::BacktraceStatus,
    collections::{BTreeMap, HashMap},
    env,
    fmt::{self, Debug, Display, Formatter},
    fs,
    path::PathBuf,
    process,
    sync::{Arc, LazyLock, atomic::AtomicBool},
    time::Instant,
  },
  subcommand::Subcommand,
  symbol::Symbol,
  tokenizer::Tokenizer,
  tokio::{io::AsyncBufReadExt, sync::RwLock},
  tokio_stream::{StreamExt, wrappers::LinesStream},
  tower_lsp::{Client, LanguageServer, LspService, jsonrpc, lsp_types as lsp},
  tree_sitter::Node,
  tree_sitter_highlight::{
    Highlight, HighlightConfiguration, HighlightEvent, Highlighter,
  },
};

mod arguments;
mod command;
mod resolver;
mod server;
mod subcommand;
mod symbol;
mod tokenizer;

type Result<T = (), E = Error> = std::result::Result<T, E>;

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
