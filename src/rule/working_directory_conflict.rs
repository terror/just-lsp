use super::*;

define_rule! {
  /// Detects conflicts between working-directory and no-cd directives.
  WorkingDirectoryConflictRule {
    id: "working-directory-conflict",
    message: "conflicting directory attributes",
    run(ctx) {
      let mut diagnostics = Vec::new();

      for recipe in ctx.recipes() {
        let working_directory_attribute =
          recipe.find_attribute("working-directory");

        let no_cd_attribute = recipe.find_attribute("no-cd");

        if let (Some(attribute), Some(_)) =
          (working_directory_attribute, no_cd_attribute)
        {
          diagnostics.push(Diagnostic::error(
            format!(
              "Recipe `{}` can't combine `[working-directory]` with `[no-cd]`",
              recipe.name
            ),
            attribute.range,
          ));
        }
      }

      diagnostics
    }
  }
}
