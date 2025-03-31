use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum OsGroup {
  Windows,
  UnixMacOS,
  LinuxOpenBSD,
  None,
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

    matches!((self, other), (OsGroup::None, _) | (_, OsGroup::None))
  }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Attribute {
  pub(crate) name: TextNode,
  pub(crate) arguments: Vec<TextNode>,
  pub(crate) range: lsp::Range,
}
