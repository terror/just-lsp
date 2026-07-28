use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ProjectDependencyKind {
  Import {
    attributes: Vec<Attribute>,
    optional: bool,
  },
}
