use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeTarget {
  Alias,
  Assignment,
  Module,
  Recipe,
}

impl Display for AttributeTarget {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        AttributeTarget::Alias => "alias",
        AttributeTarget::Assignment => "assignment",
        AttributeTarget::Module => "module",
        AttributeTarget::Recipe => "recipe",
      }
    )
  }
}

impl AttributeTarget {
  pub fn try_from_kind(kind: &str) -> Option<Self> {
    match kind {
      "alias" => Some(Self::Alias),
      "assignment" | "export" => Some(Self::Assignment),
      "module" => Some(Self::Module),
      "recipe" => Some(Self::Recipe),
      _ => None,
    }
  }
}
