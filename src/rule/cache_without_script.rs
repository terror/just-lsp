use super::*;

define_rule! {
  CacheWithoutScriptRule {
    id: "cache-without-script",
    title: "cache without script mode",
    severity: Error,
  }
}

impl CacheWithoutScriptRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let default_script = context.module().setting_enabled("default-script");

    let mut diagnostics = Vec::new();

    for recipe in context.local().recipes() {
      let Some(cache_attribute) = recipe.find_attribute("cache") else {
        continue;
      };

      if recipe.has_attribute("script") || recipe.runs_as_script(default_script)
      {
        continue;
      }

      diagnostics.push(Diagnostic::new(
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

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_cache_without_script() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "[cache]\nfoo:\n")
      .run();
  }

  #[test]
  fn cache_with_default_script() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      set default-script

      [cache]
      build:
        cargo build
      "
    })
    .run();
  }

  #[test]
  fn cache_with_default_script_and_shell() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      set default-script

      [shell]
      [cache]
      build:
        cargo build
      "
    })
    .error(
      "Recipe `build` uses `[cache]` without script mode",
      lsp::Range::at(5, 0, 6, 0),
    )
    .run();
  }

  #[test]
  fn cache_with_explicit_script() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      [script]
      [cache]
      build:
        cargo build
      "
    })
    .run();
  }

  #[test]
  fn cache_with_shebang() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      [cache]
      build:
        #!/usr/bin/env sh
        cargo build
      "
    })
    .run();
  }

  #[test]
  fn cache_with_shebang_and_shell() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      [shell]
      [cache]
      build:
        #!/usr/bin/env sh
        cargo build
      "
    })
    .error(
      "Recipe `build` uses `[cache]` without script mode",
      lsp::Range::at(3, 0, 4, 0),
    )
    .run();
  }

  #[test]
  fn cache_with_shell() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      [cache]
      build:
        cargo build
      "
    })
    .error(
      "Recipe `build` uses `[cache]` without script mode",
      lsp::Range::at(2, 0, 3, 0),
    )
    .run();
  }
}
