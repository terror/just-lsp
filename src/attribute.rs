use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum OsGroup {
  Windows,      // Windows OS
  UnixMacOS,    // Unix or macOS
  LinuxOpenBSD, // Linux or OpenBSD
  None,         // No OS attribute
}

impl OsGroup {
  pub(crate) fn from_attribute(attr_name: &str) -> Option<Self> {
    match attr_name {
      "windows" => Some(OsGroup::Windows),
      "unix" | "macos" => Some(OsGroup::UnixMacOS),
      "linux" | "openbsd" => Some(OsGroup::LinuxOpenBSD),
      _ => None,
    }
  }

  pub(crate) fn conflicts_with(&self, other: &OsGroup) -> bool {
    if self == other {
      return true;
    }

    match (self, other) {
      (OsGroup::None, _) | (_, OsGroup::None) => true,
      _ => false,
    }
  }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Attribute {
  pub(crate) name: TextNode,
  pub(crate) arguments: Vec<TextNode>,
  pub(crate) range: lsp::Range,
}
