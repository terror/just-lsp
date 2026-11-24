use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttributeKeyword<'a> {
  pub expression: AttributeExpressionKind,
  pub name: &'a str,
  pub scope: AttributeScope,
  pub value_required: bool,
}
