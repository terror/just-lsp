use super::*;

define_rule! {
  CacheWithoutScriptRule {
    id: "cache-without-script",
    message: "cache without script",
    run(context) {
      let mut diagnostics = Vec::new();

      for recipe in context.recipes() {
        let Some(cache_attribute) = recipe.find_attribute("cache") else {
          continue;
        };

        if recipe.has_attribute("script") {
          continue;
        }

        diagnostics.push(Diagnostic::error(
          format!(
            "Recipe `{}` uses `[cache]` without `[script]`",
            recipe.name.value
          ),
          cache_attribute.range,
        ));
      }

      diagnostics
    }
  }
}
