use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Binding {
  Builtin(&'static Builtin<'static>),
  Function(String),
  FunctionParameter(TextNode),
  Parameter(Parameter),
  Recipe(Namepath),
  Unresolved { suggestion: Option<String> },
  Variable(String),
}

impl Binding {
  pub(crate) fn is_local(&self) -> bool {
    matches!(self, Self::FunctionParameter(_) | Self::Parameter(_))
  }

  pub(crate) fn refers_to(
    &self,
    target: &Self,
    definition: Option<&lsp::Location>,
    locate: impl FnOnce(&Self) -> Option<lsp::Location>,
  ) -> bool {
    let same_name = match (self, target) {
      (Self::Recipe(candidate), Self::Recipe(target)) => {
        candidate.name().value == target.name().value
      }
      (Self::Function(candidate), Self::Function(target))
      | (Self::Variable(candidate), Self::Variable(target)) => {
        candidate == target
      }
      _ => return self == target,
    };

    same_name && definition.is_some() && locate(self).as_ref() == definition
  }
}
