use super::*;

define_rule! {
  /// Detects conflicts between working-directory and no-cd directives.
  WorkingDirectoryConflictRule {
    id: "working-directory-conflict",
    message: "conflicting working directory configuration",
    run(context) {
      let mut diagnostics = context.conflicting_settings(|left, right| {
        match (left.name.value.as_str(), right.name.value.as_str()) {
          ("working-directory", "no-cd") => {
            matches!(right.kind, SettingKind::Boolean(true))
          }
          ("no-cd", "working-directory") => {
            matches!(left.kind, SettingKind::Boolean(true))
          }
          _ => false,
        }
      });

      diagnostics.extend(
        context.conflicting_recipe_attributes("working-directory", "no-cd"),
      );

      diagnostics
    }
  }
}
