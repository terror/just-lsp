#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeExpressionKind {
  Any,
  Const,
  StringLiteral,
}
