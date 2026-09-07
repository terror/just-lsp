use super::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Attribute {
  pub arguments: Vec<TextNode>,
  pub name: TextNode,
  pub range: lsp::Range,
  pub target: Option<AttributeTarget>,
}

impl Attribute {
  pub(crate) fn condition(&self) -> Option<bool> {
    match self.name.value.as_str() {
      "android" | "dragonfly" | "freebsd" | "linux" | "macos" | "netbsd"
      | "openbsd" | "windows" => Some(self.name.value == env::consts::OS),
      "unix" => Some(cfg!(unix)),
      _ => None,
    }
  }
}
