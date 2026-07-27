use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeTarget {
  Alias,
  Assignment,
  Function,
  Import,
  Module,
  Recipe,
  Setting,
  Unexport,
}

impl Display for AttributeTarget {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        AttributeTarget::Alias => "alias",
        AttributeTarget::Assignment => "assignment",
        AttributeTarget::Function => "function",
        AttributeTarget::Import => "import",
        AttributeTarget::Module => "module",
        AttributeTarget::Recipe => "recipe",
        AttributeTarget::Setting => "setting",
        AttributeTarget::Unexport => "unexport",
      }
    )
  }
}

impl AttributeTarget {
  pub const ALL: &[Self] = &[
    Self::Alias,
    Self::Assignment,
    Self::Function,
    Self::Import,
    Self::Module,
    Self::Recipe,
    Self::Setting,
    Self::Unexport,
  ];

  #[must_use]
  pub fn target_name(self) -> &'static str {
    match self {
      Self::Alias => "Alias",
      Self::Assignment => "Assignment",
      Self::Function => "Function",
      Self::Import => "Import",
      Self::Module => "Module",
      Self::Recipe => "Recipe",
      Self::Setting => "Setting",
      Self::Unexport => "Unexport",
    }
  }

  #[must_use]
  pub fn try_from_kind(kind: &str) -> Option<Self> {
    match kind {
      "alias" => Some(Self::Alias),
      "assignment" | "export" => Some(Self::Assignment),
      "function_definition" => Some(Self::Function),
      "import" => Some(Self::Import),
      "module" => Some(Self::Module),
      "recipe" => Some(Self::Recipe),
      "setting" => Some(Self::Setting),
      "unexport" => Some(Self::Unexport),
      _ => None,
    }
  }
}
