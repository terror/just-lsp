use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum DependencyKind {
  Import {
    attributes: Vec<Attribute>,
    optional: bool,
  },
}
