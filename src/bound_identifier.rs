use super::*;

#[derive(Debug)]
pub(crate) struct BoundIdentifier {
  pub(crate) binding: Binding,
  pub(crate) declaration: bool,
  pub(crate) expression: bool,
  pub(crate) identifier: TextNode,
}
