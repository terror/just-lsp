use super::*;

define_rule! {
  CacheWithoutScriptRule {
    id: "cache-without-script",
    message: "cache without script mode",
    run(context) {
      let mut diagnostics = Vec::new();

      for (recipe, groups) in context.recipes_with_groups() {
        let Some(cache_attribute) = recipe.find_attribute("cache") else {
          continue;
        };

        if recipe.has_attribute("script")
          || (!recipe.has_attribute("shell")
            && (recipe.shebang.is_some()
              || context.setting_enabled_for("default-script", groups)))
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
