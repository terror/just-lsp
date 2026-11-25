use super::*;

define_rule! {
  /// Reports recipes that use the `[extension]` attribute without `[script]` or
  /// a shebang.
  ExtensionWithoutScriptRule {
    id: "extension-without-script",
    title: "extension without script",
    severity: Error,
  }
}

impl ExtensionWithoutScriptRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let default_script = context.module().setting_enabled("default-script");

    let mut diagnostics = Vec::new();

    for recipe in context.local().recipes() {
      let Some(extension_attribute) = recipe.find_attribute("extension") else {
        continue;
      };

      if recipe.runs_as_script(default_script) {
        continue;
      }

      diagnostics.push(Diagnostic::new(
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

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_ignores_imported_extension_without_script() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "[extension: 'foo']\nfoo:\n")
      .run();
  }

  #[test]
  fn extension_with_script_is_allowed() {
    AnalyzerTest::new(indoc! {
      "
      [script]
      [extension: '.sh']
      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn extension_with_shebang_is_allowed() {
    AnalyzerTest::new(indoc! {
      "
      [extension: '.sh']
      foo:
        #!/usr/bin/env bash
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn extension_without_script_or_shebang() {
    AnalyzerTest::new(indoc! {
      "
      [extension: '.sh']
      foo:
        echo \"foo\"
      "
    })
    .error(
      "Recipe `foo` uses `[extension]` without `[script]` or a shebang",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }
}
