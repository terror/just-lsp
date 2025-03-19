#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AttributeTarget {
  Recipe,
  Module,
  Alias,
  Variable,
  Any,
}

impl AttributeTarget {
  pub(crate) fn is_valid_for(&self, target: AttributeTarget) -> bool {
    *self == AttributeTarget::Any || *self == target
  }

  pub(crate) fn as_str(&self) -> &'static str {
    match self {
      AttributeTarget::Recipe => "recipe",
      AttributeTarget::Module => "module",
      AttributeTarget::Alias => "alias",
      AttributeTarget::Variable => "variable",
      AttributeTarget::Any => "any",
    }
  }
}

#[derive(Debug)]
pub(crate) struct Attribute<'a> {
  pub(crate) name: &'a str,
  pub(crate) description: &'a str,
  pub(crate) version: &'a str,
  pub(crate) target: AttributeTarget,
  pub(crate) parameters: Option<&'a str>,
}
