use super::*;

pub(crate) enum Symbol {
  Builtin(&'static Builtin<'static>),
  Function(Located<Function>),
  FunctionParameter(TextNode),
  Parameter(Parameter),
  Recipe(Located<Recipe>),
  Variable(Located<Variable>),
}
