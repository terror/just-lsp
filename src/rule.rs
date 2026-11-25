use super::*;

macro_rules! define_rule {
  (
    $(#[$doc:meta])*
    $name:ident {
      id: $id:literal,
      title: $title:literal,
      severity: $severity:ident
      $(,)?
    }
  ) => {
    $(#[$doc])*
    struct $name;

    impl Rule for $name {
      fn id(&self) -> &'static str {
        $id
      }

      fn run(&self, context: &RuleContext<'_>) -> Vec<Diagnostic> {
        Self::run(context)
      }

      fn severity(&self) -> lsp::DiagnosticSeverity {
        define_rule!(@severity $severity)
      }

      fn title(&self) -> &'static str {
        $title
      }
    }

    inventory::submit!(&$name as &dyn Rule);
  };
  (@severity Error) => {
    lsp::DiagnosticSeverity::ERROR
  };
  (@severity Warning) => {
    lsp::DiagnosticSeverity::WARNING
  };
}

inventory::collect!(&'static dyn Rule);

mod alias_recipe_conflict;
mod ambiguous_module_file;
mod assignment_unexport_conflict;
mod backtick_shebang;
mod cache_without_script;
mod circular_import;
mod deprecated_function;
mod deprecated_setting;
mod dotenv_command_conflict;
mod duplicate_alias;
mod duplicate_attribute;
mod duplicate_dependency;
mod duplicate_function;
mod duplicate_function_parameter;
mod duplicate_recipe;
mod duplicate_recipe_parameter;
mod duplicate_setting;
mod duplicate_unexports;
mod duplicate_variables;
mod exit_message_conflict;
mod extension_without_script;
mod extraneous_attribute;
mod guard_infallible_conflict;
mod inconsistent_indentation;
mod ineffective_parallel_attribute;
mod invalid_arg_attribute;
mod invalid_attribute_argument_count;
mod invalid_attribute_argument_expression;
mod invalid_attribute_keyword;
mod invalid_attribute_placement;
mod invalid_continue_signal;
mod invalid_dependency_argument_count;
mod invalid_function_argument_count;
mod invalid_import_path;
mod invalid_mapped_dependency;
mod invalid_recipe_parameter_order;
mod invalid_setting_type;
mod invalid_setting_value;
mod invalid_unicode_escape;
mod list_feature_gate;
mod missing_module_file;
mod mixed_indentation;
mod module_redefinition;
mod recipe_dependency_cycle;
mod script_shell_conflict;
mod syntax_error;
mod undefined_identifier;
mod unknown_attribute;
mod unknown_function;
mod unknown_setting;
mod unresolved_alias_target;
mod unresolved_dependency;
mod unstable_feature_gate;
mod unsupported_attribute_target;
mod unused_function;
mod unused_function_parameter;
mod unused_recipe_parameter;
mod unused_variable;
mod variable_dependency_cycle;
mod working_directory_conflict;

pub(crate) trait Rule: Sync {
  /// Whether the rule is enabled by its configuration.
  fn enabled(&self, config: &RuleConfig) -> bool {
    config.level() != Some(RuleLevel::Off)
  }

  /// Unique identifier for the rule.
  fn id(&self) -> &'static str;

  /// Execute the rule and return diagnostics.
  fn run(&self, context: &RuleContext<'_>) -> Vec<Diagnostic>;

  fn severity(&self) -> lsp::DiagnosticSeverity;

  /// What to show the user in the header of the diagnostics.
  fn title(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn documented_severity() {
    let categories = fs::read_dir(
      Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/features/analysis"),
    )
    .unwrap()
    .map(|entry| entry.unwrap().path())
    .filter(|path| path.is_dir())
    .collect::<Vec<_>>();

    for rule in inventory::iter::<&dyn Rule> {
      let Some(page) = categories
        .iter()
        .map(|category| category.join(format!("{}.md", rule.id())))
        .find(|page| page.is_file())
      else {
        continue;
      };

      let content = fs::read_to_string(page).unwrap();

      let severity = content
        .lines()
        .find_map(|line| line.strip_prefix("severity: "))
        .unwrap();

      let expected = match rule.severity() {
        lsp::DiagnosticSeverity::ERROR => "error",
        lsp::DiagnosticSeverity::WARNING => "warning",
        severity => panic!("unexpected severity {severity:?}"),
      };

      assert_eq!(severity, expected, "{}", rule.id());
    }
  }
}
