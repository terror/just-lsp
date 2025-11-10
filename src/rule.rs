use {
  super::*, attribute_arguments::AttributeArgumentsRule,
  attribute_invalid_target::AttributeInvalidTargetRule,
  attribute_target_support::AttributeTargetSupportRule,
  dependency_arguments::DependencyArgumentRule,
  duplicate_alias::DuplicateAliasRule, duplicate_recipes::DuplicateRecipeRule,
  duplicate_setting::DuplicateSettingRule,
  function_arguments::FunctionArgumentsRule,
  invalid_setting_kind::InvalidSettingKindRule,
  missing_dependencies::MissingDependencyRule,
  missing_recipe_for_alias::MissingRecipeForAliasRule,
  recipe_dependency_cycles::RecipeDependencyCycleRule,
  recipe_parameters::RecipeParameterRule, syntax::SyntaxRule,
  undefined_identifiers::UndefinedIdentifierRule,
  unknown_attribute::UnknownAttributeRule,
  unknown_function::UnknownFunctionRule, unknown_setting::UnknownSettingRule,
  unused_parameters::UnusedParameterRule, unused_variables::UnusedVariableRule,
};

mod attribute_arguments;
mod attribute_invalid_target;
mod attribute_target_support;
mod dependency_arguments;
mod duplicate_alias;
mod duplicate_recipes;
mod duplicate_setting;
mod function_arguments;
mod invalid_setting_kind;
mod missing_dependencies;
mod missing_recipe_for_alias;
mod recipe_dependency_cycles;
mod recipe_parameters;
mod syntax;
mod undefined_identifiers;
mod unknown_attribute;
mod unknown_function;
mod unknown_setting;
mod unused_parameters;
mod unused_variables;

pub(crate) static RULES: &[&dyn Rule] = &[
  &SyntaxRule,
  &MissingRecipeForAliasRule,
  &DuplicateAliasRule,
  &UnknownAttributeRule,
  &AttributeArgumentsRule,
  &AttributeInvalidTargetRule,
  &AttributeTargetSupportRule,
  &UnknownFunctionRule,
  &FunctionArgumentsRule,
  &RecipeParameterRule,
  &DuplicateRecipeRule,
  &RecipeDependencyCycleRule,
  &MissingDependencyRule,
  &DependencyArgumentRule,
  &UnknownSettingRule,
  &InvalidSettingKindRule,
  &DuplicateSettingRule,
  &UndefinedIdentifierRule,
  &UnusedVariableRule,
  &UnusedParameterRule,
];

pub(crate) struct UnresolvedIdentifier {
  pub(crate) name: String,
  pub(crate) range: lsp::Range,
}

#[derive(Default)]
pub(crate) struct IdentifierAnalysis {
  recipe_identifier_usage: HashMap<String, HashSet<String>>,
  unresolved_identifiers: Vec<UnresolvedIdentifier>,
  variable_usage: HashMap<String, bool>,
}

impl IdentifierAnalysis {
  fn new(context: &RuleContext<'_>) -> Self {
    let mut variable_usage = context
      .variables()
      .iter()
      .map(|variable| (variable.name.value.clone(), false))
      .collect::<HashMap<_, _>>();

    let mut recipe_identifier_usage = context
      .recipes()
      .iter()
      .map(|recipe| (recipe.name.clone(), HashSet::new()))
      .collect::<HashMap<_, _>>();

    let mut unresolved_identifiers = Vec::new();

    if let Some(tree) = context.tree() {
      let root = tree.root_node();

      for identifier in root.find_all("expression > value > identifier") {
        Self::record_identifier(
          context,
          &mut recipe_identifier_usage,
          &mut variable_usage,
          &mut unresolved_identifiers,
          identifier,
        );
      }

      for identifier in root.find_all("parameter > value > identifier") {
        Self::record_identifier(
          context,
          &mut recipe_identifier_usage,
          &mut variable_usage,
          &mut unresolved_identifiers,
          identifier,
        );
      }
    }

    Self {
      variable_usage,
      recipe_identifier_usage,
      unresolved_identifiers,
    }
  }

  fn record_identifier(
    context: &RuleContext<'_>,
    recipe_identifier_usage: &mut HashMap<String, HashSet<String>>,
    variable_usage: &mut HashMap<String, bool>,
    unresolved_identifiers: &mut Vec<UnresolvedIdentifier>,
    identifier: Node<'_>,
  ) {
    let document = context.document();
    let recipe_parameters = context.recipe_parameters();
    let value_names = context.variable_and_builtin_names();

    let recipe_name = identifier
      .get_parent("recipe")
      .as_ref()
      .and_then(|recipe_node| recipe_node.find("recipe_header > identifier"))
      .map_or_else(String::new, |identifier_node| {
        document.get_node_text(&identifier_node)
      });

    let identifier_name = document.get_node_text(&identifier);

    if let Some(recipe) = context.recipe(&recipe_name) {
      recipe_identifier_usage
        .entry(recipe.name.clone())
        .or_default()
        .insert(identifier_name.clone());

      if recipe_parameters.get(&recipe.name).is_some_and(|params| {
        params.iter().any(|param| param.name == identifier_name)
      }) {
        return;
      }
    }

    if let Some(usage) = variable_usage.get_mut(&identifier_name) {
      *usage = true;
    }

    if !value_names.contains(&identifier_name) {
      unresolved_identifiers.push(UnresolvedIdentifier {
        name: identifier_name,
        range: identifier.get_range(),
      });
    }
  }
}

pub(crate) trait Rule: Sync {
  /// Unique identifier for the rule.
  fn id(&self) -> &'static str;

  /// Human-readable name for the rule.
  fn display_name(&self) -> &'static str;

  /// Helper to annotate diagnostics with rule information.
  fn diagnostic(&self, mut diagnostic: lsp::Diagnostic) -> lsp::Diagnostic {
    diagnostic.source = Some(format!("just-lsp ({})", self.display_name()));
    diagnostic.code = Some(lsp::NumberOrString::String(self.id().to_string()));
    diagnostic
  }

  /// Execute the rule and return diagnostics.
  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic>;
}

pub(crate) struct RuleContext<'a> {
  aliases: OnceCell<Vec<Alias>>,
  document: &'a Document,
  document_variable_names: OnceCell<HashSet<String>>,
  identifier_analysis: OnceCell<IdentifierAnalysis>,
  recipe_names: OnceCell<HashSet<String>>,
  recipe_parameters: OnceCell<HashMap<String, Vec<Parameter>>>,
  recipes: OnceCell<Vec<Recipe>>,
  settings: OnceCell<Vec<Setting>>,
  variable_and_builtin_names: OnceCell<HashSet<String>>,
  variables: OnceCell<Vec<Variable>>,
}

impl<'a> RuleContext<'a> {
  pub(crate) fn new(document: &'a Document) -> Self {
    Self {
      aliases: OnceCell::new(),
      document,
      document_variable_names: OnceCell::new(),
      identifier_analysis: OnceCell::new(),
      recipe_names: OnceCell::new(),
      recipe_parameters: OnceCell::new(),
      recipes: OnceCell::new(),
      settings: OnceCell::new(),
      variable_and_builtin_names: OnceCell::new(),
      variables: OnceCell::new(),
    }
  }

  pub(crate) fn document(&self) -> &Document {
    self.document
  }

  pub(crate) fn tree(&self) -> Option<&Tree> {
    self.document.tree.as_ref()
  }

  pub(crate) fn aliases(&self) -> &[Alias] {
    self
      .aliases
      .get_or_init(|| self.document.get_aliases())
      .as_slice()
  }

  pub(crate) fn recipes(&self) -> &[Recipe] {
    self
      .recipes
      .get_or_init(|| self.document.get_recipes())
      .as_slice()
  }

  pub(crate) fn recipe(&self, name: &str) -> Option<&Recipe> {
    self.recipes().iter().find(|recipe| recipe.name == name)
  }

  pub(crate) fn settings(&self) -> &[Setting] {
    self
      .settings
      .get_or_init(|| self.document.get_settings())
      .as_slice()
  }

  pub(crate) fn variables(&self) -> &[Variable] {
    self
      .variables
      .get_or_init(|| self.document.get_variables())
      .as_slice()
  }

  pub(crate) fn recipe_names(&self) -> &HashSet<String> {
    self
      .recipe_names
      .get_or_init(|| self.recipes().iter().map(|r| r.name.clone()).collect())
  }

  pub(crate) fn recipe_parameters(&self) -> &HashMap<String, Vec<Parameter>> {
    self.recipe_parameters.get_or_init(|| {
      self
        .recipes()
        .iter()
        .map(|recipe| (recipe.name.clone(), recipe.parameters.clone()))
        .collect()
    })
  }

  pub(crate) fn document_variable_names(&self) -> &HashSet<String> {
    self.document_variable_names.get_or_init(|| {
      self
        .variables()
        .iter()
        .map(|variable| variable.name.value.clone())
        .collect()
    })
  }

  pub(crate) fn variable_and_builtin_names(&self) -> &HashSet<String> {
    self.variable_and_builtin_names.get_or_init(|| {
      let mut names = self.document_variable_names().clone();

      names.extend(builtins::BUILTINS.into_iter().filter_map(|builtin| {
        match builtin {
          Builtin::Constant { name, .. } => Some(name.to_owned()),
          _ => None,
        }
      }));

      names
    })
  }

  fn identifier_analysis(&self) -> &IdentifierAnalysis {
    self
      .identifier_analysis
      .get_or_init(|| IdentifierAnalysis::new(self))
  }

  pub(crate) fn variable_usage(&self) -> &HashMap<String, bool> {
    &self.identifier_analysis().variable_usage
  }

  pub(crate) fn recipe_identifier_usage(
    &self,
  ) -> &HashMap<String, HashSet<String>> {
    &self.identifier_analysis().recipe_identifier_usage
  }

  pub(crate) fn unresolved_identifiers(&self) -> &[UnresolvedIdentifier] {
    &self.identifier_analysis().unresolved_identifiers
  }
}
