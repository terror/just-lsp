use super::*;

define_rule! {
  UnusedFunctionRule {
    id: "unused-function",
    title: "unused function",
    severity: Warning,
  }
}

impl UnusedFunctionRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let called_functions = context.module().called_functions();

    context
      .local()
      .resolved_functions()
      .filter(|function| {
        !function.name.value.starts_with('_')
          && !called_functions.contains(&function.name.value)
      })
      .map(|function| {
        Diagnostic::new(
          format!("Function `{}` appears unused", function.name.value),
          function.name.range,
        )
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn unused_function_called_from_imported_document() {
    AnalyzerTest::new("set unstable\nfoo() := 'bar'\nimport 'foo.just'\n")
      .file("foo.just", "bar:\n  echo {{foo()}}\n")
      .run();
  }

  #[test]
  fn unused_function_calls() {
    #[track_caller]
    fn case(content: &str) {
      AnalyzerTest::new(&format!("set unstable\n{content}")).run();
    }

    case("foo() := 'bar'\nbar:\n  echo {{foo()}}\n");
    case("foo() := 'bar'\nbar() := foo()\nbaz:\n  echo {{bar()}}\n");
    case("env_var() := 'foo'\nbar:\n  echo {{env_var()}}\n");
  }

  #[test]
  fn unused_function_ignores_imported_definitions() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "foo() := 'bar'\n")
      .run();
  }

  #[test]
  fn unused_function_ignores_underscore_prefix() {
    AnalyzerTest::new("set unstable\n_foo() := 'bar'\n").run();
  }

  #[test]
  fn unused_function_reports_resolved_definition() {
    AnalyzerTest::new("set unstable\nfoo() := 'bar'\nfoo() := 'baz'\n")
      .error("Duplicate function `foo`", lsp::Range::at(2, 0, 3, 0))
      .warning("Function `foo` appears unused", lsp::Range::at(2, 0, 2, 3))
      .run();
  }

  #[test]
  fn unused_function_reports_uncalled_definition() {
    AnalyzerTest::new("set unstable\nfoo() := 'bar'\n")
      .warning("Function `foo` appears unused", lsp::Range::at(1, 0, 1, 3))
      .run();
  }

  #[test]
  fn unused_function_requires_call_syntax() {
    AnalyzerTest::new(indoc! {
      r#"
      set unstable

      foo() := 'bar'
      _bar(foo) := foo
      foo := "foo()"
      # foo()

      baz:
        echo {{foo}}
      "#
    })
    .warning("Function `foo` appears unused", lsp::Range::at(2, 0, 2, 3))
    .run();
  }
}
