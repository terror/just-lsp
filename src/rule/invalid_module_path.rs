use super::*;

define_rule! {
  InvalidModulePathRule {
    id: "invalid-module-path",
    title: "invalid module path",
    severity: Error,
  }
}

impl InvalidModulePathRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    context
      .document()
      .modules()
      .into_iter()
      .filter(ModuleDeclaration::is_dynamic)
      .filter_map(|module| module.path)
      .map(|path| {
        Diagnostic::new(
          "Expected keyword `x` but found identifier `f`",
          path.range,
        )
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn module_format_string() {
    AnalyzerTest::new("mod foo f'foo.just'\n")
      .error(
        "Expected keyword `x` but found identifier `f`",
        lsp::Range::at(0, 8, 0, 19),
      )
      .run();
  }
}
