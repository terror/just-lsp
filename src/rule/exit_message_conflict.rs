use super::*;

define_rule! {
  ExitMessageConflictRule {
    id: "exit-message-conflict",
    message: "conflicting exit message attributes",
    run(context) {
      let mut diagnostics = Vec::new();

      for recipe in context.recipes() {
        let exit_message_attribute = recipe.find_attribute("exit-message");

        let no_exit_message_attribute = recipe.find_attribute("no-exit-message");

        if let (Some(attribute), Some(_)) =
          (exit_message_attribute, no_exit_message_attribute)
        {
          diagnostics.push(Diagnostic::error(
            format!(
              "Recipe `{}` can't combine `[exit-message]` with `[no-exit-message]`",
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
