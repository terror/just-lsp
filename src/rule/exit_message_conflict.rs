use super::*;

define_rule! {
  ExitMessageConflictRule {
    id: "exit-message-conflict",
    message: "conflicting exit message attributes",
    run(context) {
      context.conflicting_recipe_attributes("exit-message", "no-exit-message")
    }
  }
}
