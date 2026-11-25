use super::*;

#[derive(Clone, Copy)]
pub(crate) struct ModuleScope<'a> {
  context: &'a RuleContext<'a>,
}

impl<'a> ModuleScope<'a> {
  pub(crate) fn aliases(self) -> &'a [Located<Alias>] {
    self.context.view.aliases()
  }

  pub(crate) fn called_functions(self) -> &'a HashSet<String> {
    self.context.usage().called_functions()
  }

  pub(crate) fn declares_module(self, path: &Namepath) -> bool {
    match self.context.view.module(path) {
      ModuleLookup::Found(view) => view
        .modules()
        .iter()
        .any(|module| module.name.value == path.name().value),
      ModuleLookup::Undeclared(_) | ModuleLookup::Unloaded => false,
    }
  }

  pub(crate) fn find_function(
    self,
    name: &str,
  ) -> Option<&'a Located<Function>> {
    self.context.view.find_function(name)
  }

  pub(crate) fn functions(self) -> &'a [Located<Function>] {
    self.context.view.functions()
  }

  pub(crate) fn modules(self) -> &'a [Located<ModuleDeclaration>] {
    self.context.view.modules()
  }

  pub(super) fn new(context: &'a RuleContext<'a>) -> Self {
    Self { context }
  }

  pub(crate) fn recipes(self) -> &'a [Located<Recipe>] {
    self.context.view.recipes()
  }

  pub(crate) fn resolve_recipe(
    self,
    path: &Namepath,
  ) -> Option<Located<Recipe>> {
    self.context.view.resolve_recipe(path)
  }

  pub(crate) fn resolved_functions(
    self,
  ) -> impl Iterator<Item = &'a Located<Function>> {
    self.context.view.resolved_functions()
  }

  pub(crate) fn resolved_recipes(
    self,
  ) -> impl Iterator<Item = &'a Located<Recipe>> {
    self.context.view.resolved_recipes()
  }

  pub(crate) fn resolved_variables(
    self,
  ) -> impl Iterator<Item = &'a Located<Variable>> {
    self.context.view.resolved_variables()
  }

  pub(crate) fn setting_enabled(self, name: &str) -> bool {
    self.context.view.setting_enabled(name)
  }

  pub(crate) fn settings(self) -> &'a [Located<Setting>] {
    self.context.view.settings()
  }

  pub(crate) fn unexports(self) -> &'a [Located<Unexport>] {
    self.context.view.unexports()
  }

  pub(crate) fn unresolved_recipe(
    self,
    path: &Namepath,
  ) -> Option<UnresolvedRecipe> {
    let module = match self.context.view.module(path) {
      ModuleLookup::Found(module) => module,
      ModuleLookup::Undeclared(segment) => {
        return Some(UnresolvedRecipe::Module(segment));
      }
      ModuleLookup::Unloaded => return None,
    };

    let name = path.name();

    if module.find_recipe(&name.value).is_some() {
      return None;
    }

    let suggestion = name
      .value
      .find_suggestion(
        module
          .recipes()
          .iter()
          .map(|recipe| recipe.name.value.as_str()),
      )
      .map(|suggestion| Namepath {
        range: path.range,
        segments: path
          .modules()
          .iter()
          .cloned()
          .chain([TextNode {
            range: name.range,
            value: suggestion,
          }])
          .collect(),
      });

    Some(UnresolvedRecipe::Recipe { suggestion })
  }

  pub(crate) fn used_variables(self) -> &'a HashSet<String> {
    self.context.usage().used_variables()
  }

  pub(crate) fn variables(self) -> &'a [Located<Variable>] {
    self.context.view.variables()
  }
}
