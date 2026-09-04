use super::*;

define_rule! {
  /// Reports recipes that use the `[extension]` attribute without `[script]` or a shebang.
  ExtensionWithoutScriptRule {
    id: "extension-without-script",
    message: "extension without script",
    run(context) {
      let default_script = context.setting_enabled("default-script");

      let mut diagnostics = Vec::new();

      for recipe in context.recipes() {
        let Some(extension_attribute) = recipe.find_attribute("extension") else {
          continue;
        };

        if recipe.runs_as_script(default_script) {
          continue;
        }

        diagnostics.push(Diagnostic::error(
          format!(
            "Recipe `{}` uses `[extension]` without `[script]` or a shebang",
            recipe.name.value
          ),
          extension_attribute.range,
        ));
      }

      diagnostics
    }
  }
}
