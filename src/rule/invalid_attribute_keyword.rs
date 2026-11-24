use super::*;

define_rule! {
  InvalidAttributeKeywordRule {
    id: "invalid-attribute-keyword",
    message: "invalid attribute keyword",
    run(context) {
      let mut diagnostics = Vec::new();

      for attribute in context.attributes() {
        let attribute_name = &attribute.name.value;

        let Some(Builtin::Attribute { signature, .. }) =
          context.builtin_attribute(attribute_name)
        else {
          continue;
        };

        let mut keywords = HashSet::new();

        for argument in &attribute.arguments {
          let AttributeArgument::Keyword { name, value, range } = argument
          else {
            if !keywords.is_empty() {
              diagnostics.push(Diagnostic::error(
                "Positional attribute arguments cannot follow keyword attribute arguments",
                argument.range(),
              ));
            }

            continue;
          };

          if !signature.keywords.is_empty() && !keywords.insert(&name.value) {
            diagnostics.push(Diagnostic::error(
              format!(
                "Duplicate `[{attribute_name}]` keyword `{}`",
                name.value
              ),
              name.range,
            ));
          }

          let Some(keyword) = signature
            .keywords
            .iter()
            .find(|keyword| keyword.name == name.value)
          else {
            diagnostics.push(Diagnostic::error(
              if signature.keywords.is_empty() {
                format!(
                  "Attribute `{attribute_name}` does not accept keyword arguments"
                )
              } else {
                format!(
                  "Unknown `[{attribute_name}]` keyword `{}`, expected one of {}",
                  name.value,
                  signature
                    .keywords
                    .iter()
                    .map(|keyword| keyword.name)
                    .collect::<Vec<_>>()
                    .join(", "),
                )
              },
              *range,
            ));

            continue;
          };

          if value.is_none() && keyword.value_required {
            diagnostics.push(Diagnostic::error(
              format!(
                "`[{attribute_name}]` keyword `{}` requires a value",
                name.value
              ),
              *range,
            ));
          }
        }
      }

      diagnostics
    }
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::{formatdoc, indoc},
  };

  #[test]
  fn arg_attribute_missing_keyword_value() {
    #[track_caller]
    fn case(keyword: &str, message: Option<&'static str>) {
      let test = AnalyzerTest::new(&formatdoc! {
        "
        [arg('foo', {keyword})]
        bar foo:
          echo {{{{foo}}}}
        "
      });

      let test = if let Some(message) = message {
        test.error(
          message,
          lsp::Range::at(0, 12, 0, 12 + u32::try_from(keyword.len()).unwrap()),
        )
      } else {
        test
      };

      test.run();
    }

    case("help", Some("`[arg]` keyword `help` requires a value"));
    case("short", None);
  }

  #[test]
  fn arg_attribute_unknown_kwarg() {
    AnalyzerTest::new(indoc! {
      "
      [arg('name', bogus='x')]
      foo name:
        echo {{name}}
      "
    })
    .error(
      "Unknown `[arg]` keyword `bogus`, expected one of flag, help, long, max, min, multiple, pattern, short, value",
    lsp::Range::at(0, 13, 0, 22))
    .run();
  }

  #[test]
  fn attribute_unsupported_keyword() {
    #[track_caller]
    fn case(attribute: &str, message: &'static str) {
      let test = AnalyzerTest::new(&formatdoc! {
        "
        [{attribute}(foo='bar')]
        foo:
          echo foo
        "
      });

      let test = if attribute == "group" {
        test.error(
          "Attribute `group` got 0 arguments but takes 1 argument",
          lsp::Range::at(0, 0, 1, 0),
        )
      } else {
        test
      };

      let start = 2 + u32::try_from(attribute.len()).unwrap();

      test
        .error(message, lsp::Range::at(0, start, 0, start + 9))
        .run();
    }

    case(
      "group",
      "Attribute `group` does not accept keyword arguments",
    );
    case(
      "private",
      "Attribute `private` does not accept keyword arguments",
    );
    case(
      "script",
      "Attribute `script` does not accept keyword arguments",
    );
  }

  #[test]
  fn attributes_keyword_structure() {
    #[track_caller]
    fn case(content: &str, errors: &[(&'static str, lsp::Range)]) {
      errors
        .iter()
        .fold(AnalyzerTest::new(content), |test, (message, range)| {
          test.error(message, *range)
        })
        .run();
    }

    case(
      "[arg('foo', long='bar', long='baz')]\nbar foo:\n  echo {{foo}}\n",
      &[(
        "Duplicate `[arg]` keyword `long`",
        lsp::Range::at(0, 24, 0, 28),
      )],
    );

    case(
      "[arg('foo', long, long)]\nbar foo:\n  echo {{foo}}\n",
      &[(
        "Duplicate `[arg]` keyword `long`",
        lsp::Range::at(0, 18, 0, 22),
      )],
    );

    case(
      "[arg('foo', long, long='bar')]\nbar foo:\n  echo {{foo}}\n",
      &[(
        "Duplicate `[arg]` keyword `long`",
        lsp::Range::at(0, 18, 0, 22),
      )],
    );

    case(
      "set unstable\n[cache(inputs='foo', inputs='bar')]\n[script]\nfoo:\n  echo bar\n",
      &[(
        "Duplicate `[cache]` keyword `inputs`",
        lsp::Range::at(1, 21, 1, 27),
      )],
    );

    case(
      "[arg(long, 'foo')]\nbar foo:\n  echo {{foo}}\n",
      &[(
        "Positional attribute arguments cannot follow keyword attribute arguments",
        lsp::Range::at(0, 11, 0, 16),
      )],
    );

    case(
      "set unstable\n[cache(inputs='foo', 'bar')]\n[script]\nfoo:\n  echo bar\n",
      &[
        (
          "Attribute `cache` got 1 argument but takes 0 arguments",
          lsp::Range::at(1, 0, 2, 0),
        ),
        (
          "Positional attribute arguments cannot follow keyword attribute arguments",
          lsp::Range::at(1, 21, 1, 26),
        ),
      ],
    );

    case(
      "[arg('foo', long)]\n[arg('bar', long)]\nbaz foo bar:\n  echo {{foo}} {{bar}}\n",
      &[],
    );
  }

  #[test]
  fn cache_attribute_invalid_keyword_and_missing_value() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      [script]
      [cache(foo='bar', inputs)]
      build:
        cargo build
      "
    })
    .error(
      "Unknown `[cache]` keyword `foo`, expected one of environment, extra, inputs, outputs",
      lsp::Range::at(3, 7, 3, 16),
    )
    .error(
      "`[cache]` keyword `inputs` requires a value",
      lsp::Range::at(3, 18, 3, 24),
    )
    .run();
  }

  #[test]
  fn cache_attribute_kwargs() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      environment := 'PATH'

      [script]
      [cache(environment=[environment], inputs='Cargo.lock', outputs='target', extra=arch())]
      build:
        cargo build
      "
    })
    .run();
  }
}
