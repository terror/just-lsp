use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum OsGroup {
  Windows,
  UnixMacOS,
  LinuxOpenBSD,
  None,
}

impl TryFrom<&str> for OsGroup {
  type Error = Error;

  fn try_from(value: &str) -> Result<Self> {
    match value {
      "windows" => Ok(OsGroup::Windows),
      "unix" | "macos" => Ok(OsGroup::UnixMacOS),
      "linux" | "openbsd" => Ok(OsGroup::LinuxOpenBSD),
      _ => Err(anyhow!("Invalid os group: {}", value)),
    }
  }
}

impl OsGroup {
  pub(crate) fn conflicts_with(&self, other: &OsGroup) -> bool {
    if self == other {
      return true;
    }

    matches!((self, other), (OsGroup::None, _) | (_, OsGroup::None))
  }
}
