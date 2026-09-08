#[derive(Debug, Clone, Copy)]
pub enum FunctionParameter<'a> {
  Optional(&'a str, Option<&'a str>),
  Required(&'a str, Option<&'a str>),
  Variadic(&'a str, Option<&'a str>),
}

impl FunctionParameter<'_> {
  pub(crate) fn snippet(self, index: usize) -> String {
    let (name, type_hint) = match self {
      Self::Optional(name, type_hint)
      | Self::Required(name, type_hint)
      | Self::Variadic(name, type_hint) => (name, type_hint),
    };

    let separator = if index == 1 { "" } else { ", " };

    let (prefix, separator) = if matches!(self, Self::Required(..)) {
      (separator, "")
    } else {
      ("", separator)
    };

    let type_hint = type_hint
      .map(|type_hint| format!(":{type_hint}"))
      .unwrap_or_default();

    let suffix = if matches!(self, Self::Variadic(..)) {
      "..."
    } else {
      ""
    };

    format!("{prefix}${{{index}:{separator}{name}{type_hint}{suffix}}}")
  }
}
