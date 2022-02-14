// std
pub(crate) use std::{collections::BTreeMap, env, process, sync::Arc};

// dependencies
pub(crate) use {
  lsp_text::RopeExt,
  lspower::{jsonrpc, lsp, Client, LanguageServer, LspService},
  ropey::Rope,
};

// structs and enums
pub(crate) use crate::{document::Document, message::Message, server::Server};

// type aliases
pub(crate) type Result<T = (), E = Box<dyn std::error::Error>> =
  std::result::Result<T, E>;
pub(crate) type Documents = BTreeMap<lsp::Url, Document>;
