use super::*;

define_rule! {
  UnusedFunctionParameterRule {
    id: "unused-function-parameter",
    title: "unused function parameter",
    severity: Warning,
  }
}

impl UnusedFunctionParameterRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let document = context.document();

    context
      .local()
      .tree()
      .root_node()
      .find_all("function_parameters > identifier")
      .into_iter()
      .filter(|parameter| !parameter.is_missing())
      .filter_map(|parameter| {
        let body = parameter
          .get_parent("function_definition")?
          .child_by_field_name("body")
          .filter(|body| !body.is_missing())?;

        let name = document.get_node_text(&parameter);

        let used = body
          .find_all("value > identifier")
          .iter()
          .any(|identifier| document.get_node_text(identifier) == name);

        (!name.starts_with('_') && !used).then(|| {
          Diagnostic::new(
            format!("Function parameter `{name}` appears unused"),
            document.get_range(&parameter),
          )
        })
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn unused_function_parameter_ignores_imported_definitions() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "foo(bar) := 'baz'\n")
      .run();
  }

  #[test]
  fn unused_function_parameter_reports_multiple() {
    AnalyzerTest::new("set unstable\n_foo(bar, baz, qux) := baz\n")
      .warning(
        "Function parameter `bar` appears unused",
        lsp::Range::at(1, 5, 1, 8),
      )
      .warning(
        "Function parameter `qux` appears unused",
        lsp::Range::at(1, 15, 1, 18),
      )
      .run();
  }

  #[test]
  fn unused_function_parameter_reports_unused() {
    #[track_caller]
    fn case(content: &str) {
      AnalyzerTest::new(&format!("set unstable\n{content}\n"))
        .warning(
          "Function parameter `bar` appears unused",
          lsp::Range::at(1, 5, 1, 8),
        )
        .run();
    }

    case("_foo(bar) := 'bar'");
    case("_foo(bar) := bar_suffix\nbar_suffix := 'baz'");
    case("_foo(bar) := bar()\nbar() := 'baz'");
    case("_foo(bar) := 'baz'\n_baz(bar) := bar");
  }

  #[test]
  fn unused_function_parameter_uses() {
    #[track_caller]
    fn case(content: &str) {
      AnalyzerTest::new(&format!("set unstable\n{content}\n")).run();
    }

    case("_foo() := 'bar'");
    case("_foo(bar) := bar");
    case("_foo(bar) := f'{{bar}}'");
    case("_foo(bar) := uppercase(bar)");
    case("_foo(_bar, baz) := baz");
  }
}
