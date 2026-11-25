use super::*;

define_rule! {
  InvalidAttributeArgumentExpressionRule {
    id: "invalid-attribute-argument-expression",
    title: "invalid attribute argument expression",
    severity: Error,
  }
}

impl InvalidAttributeArgumentExpressionRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for attribute in context.attributes() {
      let attribute_name = &attribute.name.value;

      let Some(Builtin::Attribute { signature, .. }) =
        Builtin::attribute(attribute_name)
      else {
        continue;
      };

      for argument in &attribute.arguments {
        let (kind, expression) = match argument {
          AttributeArgument::Positional(expression) => {
            (signature.expression, expression)
          }
          AttributeArgument::Keyword {
            name,
            value: Some(value),
            ..
          } => {
            let Some(keyword) = signature
              .keywords
              .iter()
              .find(|keyword| keyword.name == name.value)
            else {
              continue;
            };

            (keyword.expression, value)
          }
          AttributeArgument::Keyword { value: None, .. } => continue,
        };

        let expected = match (kind, expression.kind) {
          (AttributeExpressionKind::Const, AttributeExpressionKind::Any) => {
            "const expressions"
          }
          (
            AttributeExpressionKind::StringLiteral,
            AttributeExpressionKind::Any | AttributeExpressionKind::Const,
          ) => "string literals",
          _ => continue,
        };

        diagnostics.push(Diagnostic::new(
          format!("Attribute `{attribute_name}` arguments must be {expected}"),
          expression.text.range,
        ));
      }
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::{formatdoc, indoc},
  };

  #[test]
  fn arg_attribute_invalid_parameter_expression() {
    #[track_caller]
    fn case(attribute: &str, range: lsp::Range) {
      AnalyzerTest::new(&formatdoc! {
        "
        {attribute}
        bar foo:
          echo {{{{foo}}}}
        "
      })
      .error("Attribute `arg` arguments must be string literals", range)
      .run();
    }

    case("[arg('foo' + 'bar')]", lsp::Range::at(0, 5, 0, 18));
    case("[arg(arch())]", lsp::Range::at(0, 5, 0, 11));
    case("[arg: foo]", lsp::Range::at(0, 6, 0, 9));
  }

  #[test]
  fn arg_attribute_string_kwargs_reject_expressions() {
    #[track_caller]
    fn case(keyword: &'static str) {
      let content = formatdoc! {
        "
        foo := 'foo'

        [arg('bar', {keyword}=foo)]
        bar bar:
          echo {{{{bar}}}}
        "
      };

      let start = u32::try_from(13 + keyword.len()).unwrap();

      AnalyzerTest::new(&content)
        .error(
          "Attribute `arg` arguments must be string literals",
          lsp::Range::at(2, start, 2, start + 3),
        )
        .run();
    }

    case("long");
    case("short");
  }

  #[test]
  fn attribute_expression_requires_support() {
    AnalyzerTest::new(indoc! {
      "
      foo := 'foo'
      bar := 'bar'

      [group(foo)]
      [group: bar]
      [group(f'baz')]
      foo:
        echo \"foo\"
      "
    })
    .error(
      "Attribute `group` arguments must be string literals",
      lsp::Range::at(3, 7, 3, 10),
    )
    .error(
      "Attribute `group` arguments must be string literals",
      lsp::Range::at(4, 8, 4, 11),
    )
    .error(
      "Attribute `group` arguments must be string literals",
      lsp::Range::at(5, 7, 5, 13),
    )
    .run();
  }

  #[test]
  fn attribute_string_literal_expression() {
    AnalyzerTest::new(indoc! {
      "
      [group('foo')]
      [metadata(x'bar')]
      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_duplicate_group_nonliteral_arguments() {
    #[track_caller]
    fn case(expression: &str) {
      let end = 7 + u32::try_from(expression.len()).unwrap();

      AnalyzerTest::new(&format!(
        "[group({expression})]\n[group({expression})]\nfoo:\n"
      ))
      .error(
        "Attribute `group` arguments must be string literals",
        lsp::Range::at(0, 7, 0, end),
      )
      .error(
        "Attribute `group` arguments must be string literals",
        lsp::Range::at(1, 7, 1, end),
      )
      .run();
    }

    case("f'foo'");
    case("env('foo')");
    case("'foo' + 'bar'");
  }

  #[test]
  fn confirm_attribute_accepts_expression() {
    AnalyzerTest::new(indoc! {
      r#"
      [confirm("Deploy to " + env + "?")]
      deploy env:
        echo {{env}}
      "#
    })
    .run();
  }

  #[test]
  fn doc_attribute_rejects_non_const_expression() {
    AnalyzerTest::new(indoc! {
      "
      [doc(error('message'))]
      foo:
        echo foo
      "
    })
    .error(
      "Attribute `doc` arguments must be const expressions",
      lsp::Range::at(0, 5, 0, 21),
    )
    .run();
  }

  #[test]
  fn expression_attribute_arguments() {
    AnalyzerTest::new(indoc! {
      "
      bar := 'bar'

      [confirm('foo' / bar)]
      [env('FOO', bar)]
      [working-directory('foo' / bar)]
      foo:
        echo \"foo\"

      [confirm: 'foo' / bar]
      [working-directory: 'foo' / bar]
      baz:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn timestamp_attribute_accepts_optional_expression() {
    AnalyzerTest::new(indoc! {
      "
      format := '%H:%M:%S'

      [timestamp]
      foo:
        echo foo

      [timestamp(format)]
      bar:
        echo bar
      "
    })
    .run();
  }
}
