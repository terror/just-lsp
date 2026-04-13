#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Group {
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

  #[must_use]
  pub fn targets(attribute: &str) -> Option<Vec<Self>> {
    match attribute {
      "dragonfly" => Some(vec![Group::Dragonfly]),
      "freebsd" => Some(vec![Group::Freebsd]),
      "linux" => Some(vec![Group::Linux]),
      "macos" => Some(vec![Group::Macos]),
      "netbsd" => Some(vec![Group::Netbsd]),
      "openbsd" => Some(vec![Group::Openbsd]),
      "unix" => Some(vec![
        Group::Dragonfly,
        Group::Freebsd,
        Group::Linux,
        Group::Macos,
        Group::Netbsd,
        Group::Openbsd,
      ]),
      "windows" => Some(vec![Group::Windows]),
      _ => None,
    }
  }
}
