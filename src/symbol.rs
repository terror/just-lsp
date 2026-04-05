use super::*;

pub(crate) enum Symbol {
  Builtin(&'static Builtin<'static>),
  Function(Function),
  Parameter(Parameter),
  Recipe(Recipe),
  Variable(Variable),
}
