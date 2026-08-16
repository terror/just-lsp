#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Group {
  Android,
  Any,
  Dragonfly,
  Freebsd,
  Linux,
  Macos,
  Netbsd,
  Openbsd,
  Windows,
}

impl Group {
  #[must_use]
  pub fn conflicts_with(self, other: Group) -> bool {
    matches!((self, other), (Group::Any, _) | (_, Group::Any)) || self == other
  }
}
