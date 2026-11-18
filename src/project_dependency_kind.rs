#[derive(Clone, Debug, PartialEq)]
pub enum ProjectDependencyKind {
  Import,
  Module { name: String },
}
