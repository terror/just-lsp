use super::*;

#[derive(Debug, Clone, Copy)]
pub enum AttributeKind {
  Binary,
  Nullary,
  Optional,
  Unary,
  UnaryPlus,
  Variadic,
}

impl AttributeKind {
  #[must_use]
  pub fn argument_range(self) -> RangeInclusive<usize> {
    match self {
      Self::Binary => 2..=2,
      Self::Nullary => 0..=0,
      Self::Optional => 0..=1,
      Self::Unary => 1..=1,
      Self::UnaryPlus => 1..=usize::MAX,
      Self::Variadic => 0..=usize::MAX,
    }
  }
}
