use super::*;

/// Reports recipes that combine a shebang line with the `[script]` attribute.
pub(crate) struct ScriptShebangConflictRule;

impl Rule for ScriptShebangConflictRule {
  fn id(&self) -> &'static str {
    "script-shebang-conflict"
  }

  fn message(&self) -> &'static str {
    "shebang conflict"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for recipe in context.recipes() {
      let Some(script_attribute) = recipe.find_attribute("script") else {
        continue;
      };

      if recipe.shebang.is_none() {
        continue;
      }

      diagnostics.push(Diagnostic::error(
        format!(
          "Recipe `{}` has both shebang line and `[script]` attribute",
          recipe.name
        ),
        script_attribute.range,
      ));
    }

    diagnostics
  }
}
