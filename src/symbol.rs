use super::*;

pub(crate) enum Symbol {
  Builtin(&'static Builtin<'static>),
  Parameter(Parameter),
  Recipe(Recipe),
  Variable(Variable),
}
