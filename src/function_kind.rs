use super::*;

#[derive(Debug, Clone, Copy)]
pub enum FunctionKind {
  Binary,
  BinaryPlus,
  Nullary,
  Ternary,
  Unary,
  UnaryOpt,
  UnaryPlus,
}

impl FunctionKind {
  #[must_use]
  pub fn argument_range(self) -> RangeInclusive<usize> {
    match self {
      Self::Binary => 2..=2,
      Self::BinaryPlus => 2..=usize::MAX,
      Self::Nullary => 0..=0,
      Self::Ternary => 3..=3,
      Self::Unary => 1..=1,
      Self::UnaryOpt => 1..=2,
      Self::UnaryPlus => 1..=usize::MAX,
    }
  }
}
