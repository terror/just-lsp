use super::*;

pub(crate) use {
  alias_recipe_conflict::AliasRecipeConflictRule,
  attribute_arguments::AttributeArgumentsRule,
  attribute_invalid_target::AttributeInvalidTargetRule,
  attribute_target_support::AttributeTargetSupportRule,
  dependency_arguments::DependencyArgumentRule,
  duplicate_alias::DuplicateAliasRule,
  duplicate_attribute::DuplicateAttributeRule,
  duplicate_recipes::DuplicateRecipeRule,
  duplicate_setting::DuplicateSettingRule,
  duplicate_variables::DuplicateVariableRule,
  function_arguments::FunctionArgumentsRule,
  inconsistent_indentation::InconsistentIndentationRule,
  invalid_setting_kind::InvalidSettingKindRule,
  missing_dependencies::MissingDependencyRule,
  missing_recipe_for_alias::MissingRecipeForAliasRule,
  mixed_indentation::MixedIndentationRule,
  parallel_dependencies::ParallelDependenciesRule,
  recipe_dependency_cycles::RecipeDependencyCycleRule,
  recipe_parameters::RecipeParameterRule,
  script_shebang_conflict::ScriptShebangConflictRule, syntax::SyntaxRule,
  undefined_identifiers::UndefinedIdentifierRule,
  unknown_attribute::UnknownAttributeRule,
  unknown_function::UnknownFunctionRule, unknown_setting::UnknownSettingRule,
  unused_parameters::UnusedParameterRule, unused_variables::UnusedVariableRule,
  working_directory_conflict::WorkingDirectoryConflictRule,
};

mod alias_recipe_conflict;
mod attribute_arguments;
mod attribute_invalid_target;
mod attribute_target_support;
mod dependency_arguments;
mod duplicate_alias;
mod duplicate_attribute;
mod duplicate_recipes;
mod duplicate_setting;
mod duplicate_variables;
mod function_arguments;
mod inconsistent_indentation;
mod invalid_setting_kind;
mod missing_dependencies;
mod missing_recipe_for_alias;
mod mixed_indentation;
mod parallel_dependencies;
mod recipe_dependency_cycles;
mod recipe_parameters;
mod script_shebang_conflict;
mod syntax;
mod undefined_identifiers;
mod unknown_attribute;
mod unknown_function;
mod unknown_setting;
mod unused_parameters;
mod unused_variables;
mod working_directory_conflict;

pub(crate) trait Rule: Sync {
  /// Helper to annotate diagnostics with rule information.
  fn diagnostic(&self, diagnostic: lsp::Diagnostic) -> lsp::Diagnostic {
    lsp::Diagnostic {
      code: Some(lsp::NumberOrString::String(self.id().to_string())),
      source: Some(format!("just-lsp ({})", self.message())),
      ..diagnostic
    }
  }

  /// Unique identifier for the rule.
  fn id(&self) -> &'static str;

  /// What to show the user in the header of the diagnostics.
  fn message(&self) -> &'static str;

  /// Execute the rule and return diagnostics.
  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic>;
}
