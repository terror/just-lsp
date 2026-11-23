#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ProjectDependencyKind {
  Import,
  Module { name: String },
}
