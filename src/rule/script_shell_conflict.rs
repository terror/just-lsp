use super::*;

define_rule! {
  ScriptShellConflictRule {
    id: "script-shell-conflict",
    message: "conflicting script attributes",
    run(context) {
      context.conflicting_recipe_attributes("script", "shell")
    }
  }
}
