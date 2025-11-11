use super::*;

pub(crate) use {
  attribute_arguments::AttributeArgumentsRule,
  attribute_invalid_target::AttributeInvalidTargetRule,
  attribute_target_support::AttributeTargetSupportRule,
  dependency_arguments::DependencyArgumentRule,
  duplicate_alias::DuplicateAliasRule,
  duplicate_default_attribute::DuplicateDefaultAttributeRule,
  duplicate_recipes::DuplicateRecipeRule,
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
mod duplicate_default_attribute;
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

pub(crate) trait Rule: Sync {
  /// Helper to annotate diagnostics with rule information.
  fn diagnostic(&self, mut diagnostic: lsp::Diagnostic) -> lsp::Diagnostic {
    diagnostic.source = Some(format!("just-lsp ({})", self.display_name()));
    diagnostic.code = Some(lsp::NumberOrString::String(self.id().to_string()));
    diagnostic
  }

  /// Human-readable name for the rule.
  fn display_name(&self) -> &'static str;

  /// Unique identifier for the rule.
  fn id(&self) -> &'static str;

  /// Execute the rule and return diagnostics.
  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic>;
}
