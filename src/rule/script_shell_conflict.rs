use super::*;

define_rule! {
  ScriptShellConflictRule {
    id: "script-shell-conflict",
    message: "conflicting script attributes",
    run(context) {
      let mut diagnostics = Vec::new();

      for recipe in context.recipes() {
        let script_attribute = recipe.find_attribute("script");

        let shell_attribute = recipe.find_attribute("shell");

        if let (Some(attribute), Some(_)) =
          (script_attribute, shell_attribute)
        {
          diagnostics.push(Diagnostic::error(
            format!(
              "Recipe `{}` can't combine `[script]` with `[shell]`",
              recipe.name.value
            ),
            attribute.range,
          ));
        }
      }

      diagnostics
    }
  }
}
