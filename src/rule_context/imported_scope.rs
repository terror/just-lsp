use super::*;

#[derive(Clone, Copy)]
pub(crate) struct ImportedScope<'a> {
  context: &'a RuleContext<'a>,
}

impl<'a> ImportedScope<'a> {
  pub(crate) fn aliases(self) -> impl Iterator<Item = &'a Located<Alias>> {
    self.filter(self.context.view.aliases())
  }

  fn filter<T>(
    self,
    declarations: &'a [Located<T>],
  ) -> impl Iterator<Item = &'a Located<T>> {
    declarations
      .iter()
      .filter(move |declaration| !self.context.is_local(declaration))
  }

  pub(crate) fn modules(
    self,
  ) -> impl Iterator<Item = &'a Located<ModuleDeclaration>> {
    self.filter(self.context.view.modules())
  }

  pub(super) fn new(context: &'a RuleContext<'a>) -> Self {
    Self { context }
  }

  pub(crate) fn recipes(self) -> impl Iterator<Item = &'a Located<Recipe>> {
    self.filter(self.context.view.recipes())
  }

  pub(crate) fn variables(self) -> impl Iterator<Item = &'a Located<Variable>> {
    self.filter(self.context.view.variables())
  }
}
