use super::*;

macro_rules! define_rule {
  (
    $(#[$doc:meta])*
    $name:ident {
      id: $id:literal,
      message: $message:literal,
      run($context:ident) $body:block
      $(,)?
    }
  ) => {
    $(#[$doc])*
    struct $name;

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
mod arg_attribute;
mod attribute_argument_expressions;
mod attribute_invalid_target;
mod backtick_shebang;
mod cache_attribute;
mod cache_without_script;
mod continue_signals;
mod dependency_arguments;
mod deprecated_function;
mod deprecated_setting;
mod dotenv_command_conflict;
mod dotenv_path_filename_conflict;
mod duplicate_alias;
mod duplicate_attribute;
mod duplicate_dependencies;
mod duplicate_function;
mod duplicate_function_parameter;
mod duplicate_recipe;
mod duplicate_recipe_parameter;
mod duplicate_setting;
mod duplicate_unexports;
mod duplicate_variables;
mod exit_message_conflict;
mod export_unexport_conflict;
mod extension_without_script;
mod inconsistent_indentation;
mod ineffective_parallel_attribute;
mod invalid_attribute_argument_count;
mod invalid_function_argument_count;
mod invalid_import_path;
mod invalid_mapped_dependency;
mod invalid_setting_kind;
mod invalid_setting_value;
mod list_feature_gate;
mod missing_dependencies;
mod missing_recipe_for_alias;
mod mixed_indentation;
mod recipe_dependency_cycles;
mod script_shell_conflict;
mod syntax_error;
mod undefined_identifiers;
mod unknown_attribute;
mod unknown_function;
mod unknown_setting;
mod unstable_feature_gate;
mod unsupported_attribute_target;
mod unused_function;
mod unused_function_parameter;
mod unused_recipe_parameter;
mod unused_variable;
mod working_directory_conflict;

pub trait Rule: Sync {
  /// Whether the rule is enabled by its configuration.
  fn enabled(&self, config: &RuleConfig) -> bool {
    config.level() != Some(RuleLevel::Off)
  }

  /// Unique identifier for the rule.
  fn id(&self) -> &'static str;

  /// What to show the user in the header of the diagnostics.
  fn message(&self) -> &'static str;

  /// Execute the rule and return diagnostics.
  fn run(&self, context: &RuleContext<'_>) -> Vec<Diagnostic>;
}
