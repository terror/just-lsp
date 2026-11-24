use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Binding {
  Builtin(&'static Builtin<'static>),
  Function(String),
  FunctionParameter { function: lsp::Range, name: String },
  Parameter { name: String, recipe: lsp::Range },
  Recipe(Namepath),
  Unresolved { suggestion: Option<String> },
  Variable(String),
}

impl Binding {
  pub(crate) fn is_local(&self) -> bool {
    matches!(
      self,
      Self::FunctionParameter { .. } | Self::Parameter { .. }
    )
  }
}
