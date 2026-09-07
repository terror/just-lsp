use {
  anyhow::{Error, anyhow, bail},
  arguments::Arguments,
  ariadne::{Color, Label, Report, ReportKind, sources},
  clap::{Parser, builder::styling},
  command::Command,
  executor::Executor,
  just_lsp::*,
  resolver::Resolver,
  ropey::Rope,
  serde::Serialize,
  serde_json::Value,
  server::Server,
  std::{
    backtrace::BacktraceStatus,
    collections::{BTreeSet, HashMap},
    env,
    fmt::{self, Debug, Display, Formatter},
    fs,
    io::{self, stderr},
    path::PathBuf,
    process,
    sync::{LazyLock, atomic::AtomicBool},
    time::Instant,
  },
  subcommand::Subcommand,
  symbol::Symbol,
  tokenizer::Tokenizer,
  tokio::{io::AsyncBufReadExt, sync::RwLock},
  tokio_stream::{StreamExt, wrappers::LinesStream},
  tower_lsp::{Client, LanguageServer, LspService, jsonrpc, lsp_types as lsp},
  tracing::{Level, info, warn},
  tree_sitter::Node,
  tree_sitter_highlight::{
    Highlight, HighlightConfiguration, HighlightEvent, Highlighter,
  },
};

mod arguments;
mod command;
mod executor;
mod resolver;
mod server;
mod subcommand;
mod symbol;
mod tokenizer;

type Result<T = (), E = Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() {
  use yansi::Paint;

  if env::var_os("NO_COLOR").is_some() {
    yansi::disable();
  }

  tracing_subscriber::fmt()
    .with_writer(stderr)
    .with_env_filter(
      tracing_subscriber::EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy(),
    )
    .init();

  if let Err(error) = Arguments::parse().run().await {
    eprintln!("{} {error}", "error:".red().bold());

    for (i, error) in error.chain().skip(1).enumerate() {
      if i == 0 {
        eprintln!();
        eprintln!("{}", "because:".bold());
      }

      eprintln!("- {error}");
    }

    let backtrace = error.backtrace();

    if backtrace.status() == BacktraceStatus::Captured {
      eprintln!("{}", "backtrace:".bold());
      eprintln!("{backtrace}");
    }

    process::exit(1);
  }
}
