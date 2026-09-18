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
