use super::*;

pub(crate) enum ModuleLookup<'a> {
  Found(ModuleView<'a>),
  Undeclared(TextNode),
  Unloaded,
}
