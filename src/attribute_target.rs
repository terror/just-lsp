use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeTarget {
  Alias,
  Assignment,
  Module,
  Recipe,
  Setting,
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
        AttributeTarget::Setting => "setting",
      }
    )
  }
}

impl AttributeTarget {
  pub const ALL: &[Self] = &[
    Self::Alias,
    Self::Assignment,
    Self::Module,
    Self::Recipe,
    Self::Setting,
  ];

  #[must_use]
  pub fn target_name(self) -> &'static str {
    match self {
      Self::Alias => "Alias",
      Self::Assignment => "Assignment",
      Self::Module => "Module",
      Self::Recipe => "Recipe",
      Self::Setting => "Setting",
    }
  }

  #[must_use]
  pub fn try_from_kind(kind: &str) -> Option<Self> {
    match kind {
      "alias" => Some(Self::Alias),
      "assignment" | "export" => Some(Self::Assignment),
      "module" => Some(Self::Module),
      "recipe" => Some(Self::Recipe),
      "setting" => Some(Self::Setting),
      _ => None,
    }
  }
}
