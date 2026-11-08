use super::*;

#[derive(Debug, Clone, Copy)]
pub struct AttributeKeyword<'a> {
  pub expression: AttributeExpressionKind,
  pub name: &'a str,
  pub value: AttributeKeywordValue,
}

#[derive(Debug, Clone, Copy)]
pub enum AttributeKeywordValue {
  Forbidden,
  Optional,
  Required,
}
