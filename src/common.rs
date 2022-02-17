// std
pub use std::{collections::BTreeMap, env, process, sync::Arc};

// dependencies
pub use {
  lsp_text::RopeExt,
  lspower::{jsonrpc, lsp, Client, LanguageServer, LspService},
  ropey::Rope,
};

// structs and enums
pub use crate::{document::Document, message::Message, server::Server};

// type aliases
pub type Result<T = (), E = Box<dyn std::error::Error>> =
  std::result::Result<T, E>;
