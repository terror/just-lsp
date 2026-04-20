use {
  ropey::Rope,
  serde::{Deserialize, Serialize},
  std::{
    collections::{HashMap, HashSet},
    fmt::{self, Debug, Display, Formatter, Write},
    fs,
    iter::{once, successors},
    ops::ControlFlow,
    path::PathBuf,
    sync::OnceLock,
  },
  tower_lsp::lsp_types as lsp,
  tree_sitter::{InputEdit, Language, Node, Parser, Point, Tree, TreeCursor},
};

pub use {
  alias::Alias,
  analyzer::Analyzer,
  attribute::Attribute,
  attribute_target::AttributeTarget,
  builtin::Builtin,
  builtins::BUILTINS,
  config::{Config, RuleConfig, RuleLevel},
  count::Count,
  dependency::Dependency,
  diagnostic::Diagnostic,
  document::Document,
  error::Error,
  function::Function,
  function_call::FunctionCall,
  group::Group,
  import::Import,
  module::Module,
  node_ext::NodeExt,
  parameter::{Parameter, ParameterJson, ParameterKind, VariadicType},
  point_ext::PointExt,
  position_ext::PositionExt,
  quickfix::Quickfixer,
  range_ext::RangeExt,
  recipe::Recipe,
  rope_ext::{Edit, Position as RopePosition, RopeExt},
  rule::Rule,
  rule_context::RuleContext,
  scope::Scope,
  setting::{Setting, SettingKind},
  str_ext::StrExt,
  text_node::TextNode,
  variable::Variable,
};

mod alias;
mod analyzer;
mod attribute;
mod attribute_target;
mod builtin;
mod builtins;
mod config;
mod count;
mod dependency;
mod diagnostic;
mod document;
mod error;
mod function;
mod function_call;
mod group;
mod import;
mod module;
mod node_ext;
mod parameter;
mod point_ext;
mod position_ext;
mod quickfix;
mod range_ext;
mod recipe;
mod rope_ext;
mod rule;
mod rule_context;
mod scope;
mod setting;
mod str_ext;
mod text_node;
mod variable;

type Result<T = ()> = std::result::Result<T, Error>;

// SAFETY: tree_sitter_just returns a static language definition.
unsafe extern "C" {
  pub fn tree_sitter_just() -> Language;
}
