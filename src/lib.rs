use {
  binding::Binding,
  bindings::Bindings,
  bound_identifier::BoundIdentifier,
  conflict_tracker::ConflictTracker,
  count::Count,
  document_entry::DocumentEntry,
  document_store::DocumentStore,
  duplicates::Duplicates,
  filesystem::{Filesystem, OsFilesystem},
  function_parameter::FunctionParameter::{Optional, Required, Variadic},
  graph::Graph,
  group::Group,
  group_set::GroupSet,
  import_scope::ImportScope,
  indoc::indoc,
  module::Module,
  module_context::ModuleContext,
  module_lookup::ModuleLookup,
  path_ext::PathExt,
  point_ext::PointExt,
  project::Project,
  project_dependency::ProjectDependency,
  project_dependency_kind::ProjectDependencyKind,
  project_dependency_target::ProjectDependencyTarget,
  project_loader::ProjectLoader,
  recipe_line::{IndentKind, RecipeLine},
  references::References,
  ropey::Rope,
  rule::Rule,
  rule_context::RuleContext,
  scope_document::ScopeDocument,
  serde::Deserialize,
  serde_json::Value,
  std::{
    cmp::Reverse,
    collections::{
      BTreeMap, BTreeSet, HashMap, HashSet, VecDeque, hash_map::Entry,
    },
    env,
    fmt::{self, Debug, Display, Formatter},
    fs,
    hash::Hash,
    io,
    iter::{once, successors},
    mem,
    ops::{ControlFlow, Deref, RangeInclusive},
    path::{Component, Path, PathBuf},
    process, ptr,
    sync::{Arc, OnceLock},
  },
  str_ext::StrExt,
  string_delimiter::StringDelimiter,
  string_kind::StringKind,
  string_literal::StringLiteral,
  symbol_table::SymbolTable,
  tower_lsp::lsp_types as lsp,
  tracing::warn,
  tree_sitter::{InputEdit, Language, Node, Parser, Point, Tree},
  tree_walker::TreeWalker,
  url_ext::UrlExt,
};

#[cfg(test)]
use {
  filesystem::MemoryFilesystem, test_module::TestModule,
  test_project::TestProject,
};

pub use {
  alias::Alias,
  analyzer::Analyzer,
  attribute::Attribute,
  attribute_argument::AttributeArgument,
  attribute_expression::AttributeExpression,
  attribute_expression_kind::AttributeExpressionKind,
  attribute_keyword::AttributeKeyword,
  attribute_kind::AttributeKind,
  attribute_signature::AttributeSignature,
  attribute_target::AttributeTarget,
  builtin::Builtin,
  builtins::BUILTINS,
  config::{Config, FormattingConfig, RuleConfig, RuleLevel},
  dependency::Dependency,
  dependency_argument::DependencyArgument,
  dependency_phase::DependencyPhase,
  deprecation::Deprecation,
  diagnostic::Diagnostic,
  document::Document,
  error::Error,
  function::Function,
  function_call::FunctionCall,
  function_parameter::FunctionParameter,
  function_signature::FunctionSignature,
  import::Import,
  located::Located,
  module_declaration::ModuleDeclaration,
  module_view::ModuleView,
  namepath::Namepath,
  node_ext::NodeExt,
  parameter::{Parameter, ParameterKind, VariadicType},
  quickfix::Quickfix,
  range_ext::RangeExt,
  recipe::Recipe,
  resolver::Resolver,
  rope_ext::{Edit, Position as RopePosition, RopeExt},
  setting::Setting,
  setting_kind::SettingKind,
  symbol::Symbol,
  text_node::TextNode,
  unexport::Unexport,
  variable::Variable,
  workspace::Workspace,
};

mod alias;
mod analyzer;
mod attribute;
mod attribute_argument;
mod attribute_expression;
mod attribute_expression_kind;
mod attribute_keyword;
mod attribute_kind;
mod attribute_signature;
mod attribute_target;
mod binding;
mod bindings;
mod bound_identifier;
mod builtin;
mod builtins;
mod config;
mod conflict_tracker;
mod count;
mod dependency;
mod dependency_argument;
mod dependency_phase;
mod deprecation;
mod diagnostic;
mod document;
mod document_entry;
mod document_store;
mod duplicates;
mod error;
mod filesystem;
mod function;
mod function_call;
mod function_parameter;
mod function_signature;
mod graph;
mod group;
mod group_set;
mod import;
mod import_scope;
mod located;
mod module;
mod module_context;
mod module_declaration;
mod module_lookup;
mod module_view;
mod namepath;
mod node_ext;
mod parameter;
mod path_ext;
mod point_ext;
mod project;
mod project_dependency;
mod project_dependency_kind;
mod project_dependency_target;
mod project_loader;
mod quickfix;
mod range_ext;
mod recipe;
mod recipe_line;
mod references;
mod resolver;
mod rope_ext;
mod rule;
mod rule_context;
mod scope_document;
mod setting;
mod setting_kind;
mod str_ext;
mod string_delimiter;
mod string_kind;
mod string_literal;
mod symbol;
mod symbol_table;
#[cfg(test)]
mod test_module;
#[cfg(test)]
mod test_project;
mod text_node;
mod tree_walker;
mod unexport;
mod url_ext;
mod variable;
mod workspace;

type Result<T = ()> = std::result::Result<T, Error>;

// SAFETY: tree_sitter_just returns a static language definition.
unsafe extern "C" {
  pub fn tree_sitter_just() -> Language;
}
