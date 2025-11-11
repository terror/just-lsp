#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Group {
  Any,
  Linux,
  Macos,
  Openbsd,
  Windows,
}

impl Group {
  #[must_use]
  pub fn conflicts_with(&self, other: &Group) -> bool {
    matches!((self, other), (Group::Any, _) | (_, Group::Any)) || self == other
  }

  #[must_use]
  pub fn targets(attribute: &str) -> Option<Vec<Self>> {
    match attribute {
      "windows" => Some(vec![Group::Windows]),
      "linux" => Some(vec![Group::Linux]),
      "macos" => Some(vec![Group::Macos]),
      "openbsd" => Some(vec![Group::Openbsd]),
      "unix" => Some(vec![Group::Linux, Group::Macos, Group::Openbsd]),
      _ => None,
    }
  }
}
