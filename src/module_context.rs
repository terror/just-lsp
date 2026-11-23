use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ModuleContext<'a> {
  pub(crate) documents: &'a DocumentStore,
  pub(crate) module: &'a Module,
  pub(crate) project: &'a Project,
}
