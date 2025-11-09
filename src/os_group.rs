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

  pub(crate) fn targets(attribute: &str) -> Option<Vec<Self>> {
    match attribute {
      "windows" => Some(vec![OsGroup::Windows]),
      "linux" => Some(vec![OsGroup::Linux]),
      "macos" => Some(vec![OsGroup::Macos]),
      "openbsd" => Some(vec![OsGroup::Openbsd]),
      "unix" => Some(vec![OsGroup::Linux, OsGroup::Macos, OsGroup::Openbsd]),
      _ => None,
    }
  }
}
