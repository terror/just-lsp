use {
  alias::Alias,
  analyzer::Analyzer,
  anyhow::{Error, anyhow, bail},
  arguments::Arguments,
  ariadne::{Color, Label, Report, ReportKind, sources},
  attribute::Attribute,
  attribute_target::AttributeTarget,
  builtin::Builtin,
  builtins::BUILTINS,
  clap::Parser as Clap,
  command::Command,
  count::Count,
  dependency::Dependency,
  diagnostic::Diagnostic,
  document::Document,
  env_logger::Env,
  function_call::FunctionCall,
  group::Group,
  node_ext::NodeExt,
  once_cell::sync::{Lazy, OnceCell},
  parameter::{Parameter, ParameterJson, ParameterKind},
  point_ext::PointExt,
  recipe::Recipe,
  resolver::Resolver,
  rope_ext::RopeExt,
  ropey::Rope,
  rule::*,
  rule_context::RuleContext,
  serde::{Deserialize, Serialize},
  server::Server,
  setting::{Setting, SettingKind},
  std::{
    backtrace::BacktraceStatus,
    collections::{BTreeMap, HashMap, HashSet},
    env,
    fmt::{self, Debug, Display, Formatter, Write},
    fs,
    ops::ControlFlow,
    path::{Path, PathBuf},
    process,
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
  },
  str_ext::StrExt,
  subcommand::Subcommand,
  text_node::TextNode,
  tokio::{io::AsyncBufReadExt, sync::RwLock},
  tokio_stream::{StreamExt, wrappers::LinesStream},
  tower_lsp::{Client, LanguageServer, LspService, jsonrpc, lsp_types as lsp},
  tree_sitter::{InputEdit, Language, Node, Parser, Point, Tree, TreeCursor},
  tree_sitter_highlight::{
    Highlight, HighlightConfiguration, HighlightEvent, Highlighter,
  },
  variable::Variable,
};

mod alias;
mod analyzer;
mod arguments;
mod attribute;
mod attribute_target;
mod builtin;
mod builtins;
mod command;
mod count;
mod dependency;
mod diagnostic;
mod document;
mod function_call;
mod group;
mod node_ext;
mod parameter;
mod point_ext;
mod position_ext;
mod recipe;
mod resolver;
mod rope_ext;
mod rule;
mod rule_context;
mod server;
mod setting;
mod str_ext;
mod subcommand;
mod text_node;
mod tokenizer;
mod variable;

type Result<T = (), E = Error> = std::result::Result<T, E>;

unsafe extern "C" {
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
