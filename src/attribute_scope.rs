use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttributeScope {
  Globals,
  Parameters,
  PrecedingParameters,
}
