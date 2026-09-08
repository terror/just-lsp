use super::*;

#[derive(Debug, Clone, Copy)]
pub struct FunctionSignature<'a>(pub &'a [FunctionParameter<'a>]);

impl FunctionSignature<'_> {
  #[must_use]
  pub fn argument_range(self) -> RangeInclusive<usize> {
    let min = self
      .0
      .iter()
      .filter(|parameter| matches!(parameter, FunctionParameter::Required(..)))
      .count();

    let max = if self
      .0
      .iter()
      .any(|parameter| matches!(parameter, FunctionParameter::Variadic(..)))
    {
      usize::MAX
    } else {
      self.0.len()
    };

    min..=max
  }

  #[must_use]
  pub fn snippet(self, name: &str) -> String {
    let arguments = self
      .0
      .iter()
      .enumerate()
      .map(|(index, parameter)| parameter.snippet(index + 1))
      .collect::<String>();

    format!("{name}({arguments})")
  }
}
