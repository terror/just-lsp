use super::*;

define_rule! {
  CacheWithoutScriptRule {
    id: "cache-without-script",
    message: "cache without script mode",
    run(context) {
      let default_script = context.setting_enabled("default-script");

      let mut diagnostics = Vec::new();

      for recipe in context.recipes() {
        let Some(cache_attribute) = recipe.find_attribute("cache") else {
          continue;
        };

        // Script and shell conflicts are reported separately.
        if recipe.has_attribute("script")
          || recipe.runs_as_script(default_script)
        {
          continue;
        }

        diagnostics.push(Diagnostic::error(
          format!(
            "Recipe `{}` uses `[cache]` without script mode",
            recipe.name.value
          ),
          cache_attribute.range,
        ));
      }

      diagnostics
    }
  }
}
