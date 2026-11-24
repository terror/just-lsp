use super::*;

pub enum Symbol {
  Builtin(&'static Builtin<'static>),
  Function(Located<Function>),
  FunctionParameter(TextNode),
  Parameter(Parameter),
  Recipe(Located<Recipe>),
  Variable(Located<Variable>),
}

impl Symbol {
  pub(crate) fn definition(&self) -> Option<lsp::Location> {
    match self {
      Self::Function(function) => Some(function.location(function.name.range)),
      Self::Recipe(recipe) => Some(recipe.location(recipe.range)),
      Self::Variable(variable) => Some(variable.location(variable.range)),
      Self::Builtin(_) | Self::FunctionParameter(_) | Self::Parameter(_) => {
        None
      }
    }
  }

  #[must_use]
  pub fn is_renameable(&self) -> bool {
    matches!(
      self,
      Self::Function(_)
        | Self::FunctionParameter(_)
        | Self::Parameter(_)
        | Self::Recipe(_)
        | Self::Variable(_)
    )
  }
}
