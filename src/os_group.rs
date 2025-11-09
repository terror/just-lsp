use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum OsGroup {
  Any,
  Windows,
  Linux,
  Macos,
  Openbsd,
}

impl OsGroup {
  pub(crate) fn conflicts_with(&self, other: &OsGroup) -> bool {
    matches!((self, other), (OsGroup::Any, _) | (_, OsGroup::Any))
      || self == other
  }
}
