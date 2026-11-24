use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttributeSignature<'a> {
  pub expression: AttributeExpressionKind,
  pub keywords: &'a [AttributeKeyword<'a>],
  pub positional: AttributeKind,
}
