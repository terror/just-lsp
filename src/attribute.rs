use super::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Attribute {
  pub arguments: Vec<AttributeArgument>,
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

  pub(crate) fn keyword_argument(
    &self,
    name: &str,
  ) -> Option<&AttributeArgument> {
    self.arguments.iter().find(|argument| {
      matches!(argument, AttributeArgument::Keyword { name: keyword, .. } if keyword.value == name)
    })
  }

  pub(crate) fn positional_arguments(
    &self,
  ) -> impl Iterator<Item = &AttributeExpression> {
    self.arguments.iter().filter_map(|argument| match argument {
      AttributeArgument::Positional(expression) => Some(expression),
      AttributeArgument::Keyword { .. } => None,
    })
  }
}
