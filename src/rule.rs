use super::*;

macro_rules! define_rule {
  (
    $(#[$doc:meta])*
    $name:ident {
      id: $id:literal,
      message: $message:literal,
      run($context:ident) $body:block
    }
  ) => {
    $(#[$doc])*
    pub(crate) struct $name;

    impl Rule for $name {
      fn id(&self) -> &'static str {
        $id
      }

      fn message(&self) -> &'static str {
        $message
      }

      fn run(&self, $context: &RuleContext<'_>) -> Vec<Diagnostic> {
        $body
      }
    }

    inventory::submit!(&$name as &dyn Rule);
  };
}

inventory::collect!(&'static dyn Rule);

mod alias_recipe_conflict;
mod attribute_arguments;
mod attribute_invalid_target;
mod attribute_target_support;
mod dependency_arguments;
mod deprecated_function;
mod deprecated_setting;
mod duplicate_alias;
mod duplicate_attribute;
mod duplicate_dependencies;
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
  /// Unique identifier for the rule.
  fn id(&self) -> &'static str;

  /// What to show the user in the header of the diagnostics.
  fn message(&self) -> &'static str;

  /// Execute the rule and return diagnostics.
  fn run(&self, context: &RuleContext<'_>) -> Vec<Diagnostic>;
}
