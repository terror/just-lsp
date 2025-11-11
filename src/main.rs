use {
  anyhow::{anyhow, bail, Error},
  arguments::Arguments,
  ariadne::{sources, Color, Label, Report, ReportKind},
  clap::Parser as Clap,
  command::Command,
  env_logger::Env,
  just_lsp_analysis::{AnalysisHost, FileId},
  just_lsp_analyzer::Analyzer,
  just_lsp_builtins::BUILTINS,
  just_lsp_document::Document,
  just_lsp_resolver::Resolver,
  just_lsp_rope_ext::RopeExt,
  just_lsp_types::ParameterJson,
  server::Server,
  std::{
    backtrace::BacktraceStatus,
    collections::{BTreeMap, HashMap},
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
  tokio::{
    io::AsyncBufReadExt,
    sync::{Mutex, RwLock},
  },
  tokio_stream::{wrappers::LinesStream, StreamExt},
  tower_lsp::{jsonrpc, lsp_types as lsp, Client, LanguageServer, LspService},
};

mod arguments;
mod command;
mod server;
mod subcommand;

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
