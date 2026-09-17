use super::*;

#[derive(Debug, Clone, Copy)]
pub struct AttributeSignature<'a> {
  pub expression: AttributeExpressionKind,
  pub keywords: &'a [AttributeKeyword<'a>],
  pub positional: AttributeKind,
}
