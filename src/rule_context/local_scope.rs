use super::*;

#[derive(Clone, Copy)]
pub(crate) struct LocalScope<'a> {
  context: &'a RuleContext<'a>,
}

impl<'a> LocalScope<'a> {
  pub(crate) fn aliases(self) -> impl Iterator<Item = &'a Located<Alias>> {
    self.filter(self.context.view.aliases())
  }

  pub(crate) fn attributes(self) -> &'a [Attribute] {
    self
      .context
      .attributes
      .get_or_init(|| self.context.document().attributes())
      .as_slice()
  }

  fn filter<T>(
    self,
    declarations: impl IntoIterator<Item = &'a Located<T>>,
  ) -> impl Iterator<Item = &'a Located<T>>
  where
    T: 'a,
  {
    declarations
      .into_iter()
      .filter(move |declaration| self.context.is_local(declaration))
  }

  pub(crate) fn function_calls(self) -> &'a [FunctionCall] {
    self
      .context
      .function_calls
      .get_or_init(|| self.context.document().function_calls())
      .as_slice()
  }

  pub(crate) fn functions(self) -> impl Iterator<Item = &'a Located<Function>> {
    self.filter(self.context.view.functions())
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

  pub(crate) fn resolved_functions(
    self,
  ) -> impl Iterator<Item = &'a Located<Function>> {
    self.filter(self.context.view.resolved_functions())
  }

  pub(crate) fn resolved_recipes(
    self,
  ) -> impl Iterator<Item = &'a Located<Recipe>> {
    self.filter(self.context.view.resolved_recipes())
  }

  pub(crate) fn resolved_variables(
    self,
  ) -> impl Iterator<Item = &'a Located<Variable>> {
    self.filter(self.context.view.resolved_variables())
  }

  pub(crate) fn settings(self) -> impl Iterator<Item = &'a Located<Setting>> {
    self.filter(self.context.view.settings())
  }

  pub(crate) fn tree(self) -> &'a Tree {
    &self.context.document().tree
  }

  pub(crate) fn unexports(self) -> impl Iterator<Item = &'a Located<Unexport>> {
    self.filter(self.context.view.unexports())
  }

  pub(crate) fn unresolved_identifiers(
    self,
  ) -> &'a [(TextNode, Option<String>)] {
    self
      .context
      .usage()
      .unresolved_identifiers(&self.context.document().uri)
  }

  pub(crate) fn used_parameters(self) -> &'a [Parameter] {
    self
      .context
      .usage()
      .used_parameters(&self.context.document().uri)
  }

  pub(crate) fn variables(self) -> impl Iterator<Item = &'a Located<Variable>> {
    self.filter(self.context.view.variables())
  }
}
