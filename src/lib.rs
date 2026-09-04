use {
  document_entry::DocumentEntry,
  indoc::indoc,
  lexiclean::Lexiclean,
  project_view_document::ProjectViewDocument,
  ropey::Rope,
  serde::{Deserialize, Serialize},
  std::{
    cmp::Reverse,
    collections::{HashMap, HashSet, hash_map::Entry},
    fmt::{self, Debug, Display, Formatter},
    fs,
    iter::{once, successors},
    ops::{ControlFlow, Deref, RangeInclusive},
    path::{Path, PathBuf},
    process,
    sync::OnceLock,
  },
  string_delimiter::StringDelimiter,
  string_kind::StringKind,
  string_literal::StringLiteral,
  tower_lsp::lsp_types as lsp,
  tracing::warn,
  tree_sitter::{InputEdit, Language, Node, Parser, Point, Tree, TreeCursor},
};

pub use {
  alias::Alias,
  analyzer::Analyzer,
  attribute::Attribute,
  attribute_kind::AttributeKind,
  attribute_target::AttributeTarget,
  builtin::Builtin,
  builtins::BUILTINS,
  config::{Config, FormattingConfig, RuleConfig, RuleLevel},
  count::Count,
  dependency::Dependency,
  dependency_argument::DependencyArgument,
  dependency_phase::DependencyPhase,
  deprecation::Deprecation,
  diagnostic::Diagnostic,
  document::Document,
  document_store::DocumentStore,
  error::Error,
  function::Function,
  function_call::FunctionCall,
  function_kind::FunctionKind,
  group::Group,
  group_set::GroupSet,
  import::Import,
  import_scope::ImportScope,
  import_scope_document::ImportScopeDocument,
  located::Located,
  module::Module,
  node_ext::NodeExt,
  parameter::{Parameter, ParameterJson, ParameterKind, VariadicType},
  point_ext::PointExt,
  position_ext::PositionExt,
  project::Project,
  project_dependency::ProjectDependency,
  project_dependency_kind::ProjectDependencyKind,
  project_dependency_target::ProjectDependencyTarget,
  project_loader::ProjectLoader,
  project_view::ProjectView,
  quickfix::Quickfix,
  quickfixer::Quickfixer,
  range_ext::RangeExt,
  recipe::Recipe,
  rope_ext::{Edit, Position as RopePosition, RopeExt},
  rule::Rule,
  rule_context::RuleContext,
  scope::Scope,
  setting::Setting,
  setting_kind::SettingKind,
  str_ext::StrExt,
  text_node::TextNode,
  unexport::Unexport,
  variable::Variable,
  workspace::Workspace,
};

mod alias;
mod analyzer;
mod attribute;
mod attribute_kind;
mod attribute_target;
mod builtin;
mod builtins;
mod config;
mod count;
mod dependency;
mod dependency_argument;
mod dependency_phase;
mod deprecation;
mod diagnostic;
mod document;
mod document_entry;
mod document_store;
mod error;
mod function;
mod function_call;
mod function_kind;
mod group;
mod group_set;
mod import;
mod import_scope;
mod import_scope_document;
mod located;
mod module;
mod node_ext;
mod parameter;
mod point_ext;
mod position_ext;
mod project;
mod project_dependency;
mod project_dependency_kind;
mod project_dependency_target;
mod project_loader;
mod project_view;
mod project_view_document;
mod quickfix;
mod quickfixer;
mod range_ext;
mod recipe;
mod rope_ext;
mod rule;
mod rule_context;
mod scope;
mod setting;
mod setting_kind;
mod str_ext;
mod string_delimiter;
mod string_kind;
mod string_literal;
mod text_node;
mod unexport;
mod variable;
mod workspace;

type Result<T = ()> = std::result::Result<T, Error>;

// SAFETY: tree_sitter_just returns a static language definition.
unsafe extern "C" {
  pub fn tree_sitter_just() -> Language;
}
