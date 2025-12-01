use super::*;

define_rule! {
  /// Reports recipes that combine a shebang line with the `[script]` attribute.
  ScriptShebangConflictRule {
    id: "script-shebang-conflict",
    message: "shebang conflict",
    run(context) {
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
}
