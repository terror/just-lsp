use super::*;

/// Emits diagnostics when variable assignments reuse the same name without
/// explicitly opting into overriding via `allow-duplicate-variables`.
pub(crate) struct DuplicateVariableRule;

impl Rule for DuplicateVariableRule {
  fn id(&self) -> &'static str {
    "duplicate-variable"
  }

  fn message(&self) -> &'static str {
    "duplicate variable"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let allow_duplicates = context.setting_enabled("allow-duplicate-variables");

    if allow_duplicates {
      return Vec::new();
    }

    let mut diagnostics = Vec::new();
    let mut seen = HashSet::new();

    for variable in context.variables() {
      if !seen.insert(variable.name.value.clone()) {
        diagnostics.push(Diagnostic::error(
          format!("Duplicate variable `{}`", variable.name.value),
          variable.range,
        ));
      }
    }

    diagnostics
  }
}
