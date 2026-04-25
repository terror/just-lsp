use super::*;

pub(crate) enum Symbol {
  Builtin(&'static Builtin<'static>),
  Function(Function),
  FunctionParameter(TextNode),
  Parameter(Parameter),
  Recipe(Recipe),
  Variable(Variable),
}
