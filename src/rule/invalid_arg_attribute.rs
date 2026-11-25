use super::*;

static COUNT: LazyLock<Regex> =
  LazyLock::new(|| Regex::new("^(0|[1-9][0-9]*)$").unwrap());

define_rule! {
  /// Validates `[arg(NAME, ...)]` attributes: that NAME refers to an existing
  /// recipe parameter, that only known keyword arguments are used, and that
  /// `value=` is paired with `long=` or `short=`.
  InvalidArgAttributeRule {
    id: "invalid-arg-attribute",
    title: "invalid arg attribute",
    severity: Error,
  }
}

impl InvalidArgAttributeRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for attribute in context
      .attributes()
      .iter()
      .filter(|attribute| attribute.name.value == "arg")
    {
      let Some(parameter) = attribute.arg_parameter() else {
        continue;
      };

      let long = keyword(attribute, "long");
      let short = keyword(attribute, "short");

      for (key, option) in [("long", long), ("short", short)] {
        let Some((_, Some(expression))) = option else {
          continue;
        };

        let Some(name) = expression.string_value() else {
          continue;
        };

        let message = if name.contains('=') {
          Some(format!(
            "Option name for parameter `{parameter}` contains equal sign"
          ))
        } else if name.starts_with('-') {
          Some(format!(
            "Option name for parameter `{parameter}` starts with dash"
          ))
        } else if name.is_empty() {
          Some(format!("Option name for parameter `{parameter}` is empty"))
        } else if key == "short" && name.chars().count() != 1 {
          Some(format!(
            "Short option name for parameter `{parameter}` contains multiple characters"
          ))
        } else {
          None
        };

        if let Some(message) = message {
          diagnostics.push(Diagnostic::new(message, expression.text.range));
        }
      }

      let has_option = long.is_some() || short.is_some();

      let pattern = keyword(attribute, "pattern");
      let value = keyword(attribute, "value");

      if let Some((key, _)) = value
        && !has_option
      {
        diagnostics.push(Diagnostic::new(
          "Argument attribute `value` only valid with `long` or `short`",
          key.range,
        ));
      }

      if let Some((key, expression)) = keyword(attribute, "flag") {
        let message = if expression.is_some() {
          Some(format!(
            "`flag` attribute for argument `{parameter}` takes no value"
          ))
        } else if !has_option {
          Some(
            "Argument attribute `flag` only valid with `long` or `short`"
              .into(),
          )
        } else if value.is_some() {
          Some(format!(
            "Argument `{parameter}` may not have both `flag` and `value` attributes"
          ))
        } else if pattern.is_some() {
          Some(format!(
            "Argument `{parameter}` may not have both `flag` and `pattern` attributes"
          ))
        } else {
          None
        };

        if let Some(message) = message {
          diagnostics.push(Diagnostic::new(message, key.range));
        }
      }

      if let Some((key, expression)) = keyword(attribute, "multiple") {
        let message = if expression.is_some() {
          Some("Attribute key `multiple` takes no value")
        } else if !has_option {
          Some(
            "Argument attribute `multiple` only valid with `long` or `short`",
          )
        } else {
          None
        };

        if let Some(message) = message {
          diagnostics.push(Diagnostic::new(message, key.range));
        }
      }

      if let Some((key, Some(expression))) = pattern
        && let Some(pattern) = expression.string_value()
        && (Regex::new(&pattern).is_err()
          || Regex::new(&format!("^(?:{pattern})$")).is_err())
      {
        diagnostics.push(Diagnostic::new(
          "Failed to parse argument pattern",
          key.range,
        ));
      }

      let max = count(attribute, "max", &mut diagnostics);
      let min = count(attribute, "min", &mut diagnostics);

      if let (Some((_, max)), Some((key, min))) = (max, min)
        && min > max
      {
        diagnostics.push(Diagnostic::new(
          format!("Argument attribute `min` `{min}` exceeds `max` `{max}`"),
          key.range,
        ));
      }
    }

    for recipe in context.local_declarations(context.recipes()) {
      let mut seen = HashSet::new();

      let (mut longs, mut shorts) = (HashSet::new(), HashSet::new());

      for attribute in recipe
        .attributes
        .iter()
        .filter(|attribute| attribute.name.value == "arg")
      {
        let Some(name) = attribute.positional_arguments().next() else {
          continue;
        };

        let Some(parameter_name) = name.string_value() else {
          continue;
        };

        let parameter = recipe
          .parameters
          .iter()
          .find(|parameter| parameter.name == parameter_name);

        if parameter.is_none() {
          diagnostics.push(Diagnostic::new(
            format!("`[arg]` references unknown parameter `{parameter_name}`"),
            name.text.range,
          ));
        }

        if !seen.insert(parameter_name.clone()) {
          diagnostics.push(Diagnostic::new(
            format!(
              "`[arg]` attribute for parameter `{parameter_name}` is duplicated"
            ),
            attribute.range,
          ));
        }

        if let Some((long, range)) = option(attribute, "long", &parameter_name)
          && !longs.insert(long.clone())
        {
          diagnostics.push(Diagnostic::new(
            format!(
              "Recipe `{}` defines option `--{long}` multiple times",
              recipe.name.value
            ),
            range,
          ));
        }

        if let Some((short, range)) =
          option(attribute, "short", &parameter_name)
          && let Some(short) = short.chars().next()
          && !shorts.insert(short)
        {
          diagnostics.push(Diagnostic::new(
            format!(
              "Recipe `{}` defines option `-{short}` multiple times",
              recipe.name.value
            ),
            range,
          ));
        }

        let Some(parameter) = parameter else {
          continue;
        };

        if keyword(attribute, "flag").is_some()
          && parameter.default_value.is_some()
        {
          diagnostics.push(Diagnostic::new(
            format!("Flag parameter `{parameter_name}` may not have a default"),
            parameter.range,
          ));
        }

        if let Some((key, _)) =
          keyword(attribute, "max").or_else(|| keyword(attribute, "min"))
          && keyword(attribute, "multiple").is_none()
          && parameter.kind == ParameterKind::Normal
        {
          diagnostics.push(Diagnostic::new(
            format!(
              "Argument attribute `{}` only valid with `multiple` or a variadic parameter",
              key.value
            ),
            key.range,
          ));
        }
      }
    }

    diagnostics
  }
}

fn count<'a>(
  attribute: &'a Attribute,
  key: &str,
  diagnostics: &mut Vec<Diagnostic>,
) -> Option<(&'a TextNode, u64)> {
  let (name, Some(expression)) = keyword(attribute, key)? else {
    return None;
  };

  let value = expression.string_value()?;

  if !COUNT.is_match(&value) {
    diagnostics.push(Diagnostic::new(
      format!("Invalid `{key}` value `{value}`"),
      expression.text.range,
    ));

    return None;
  }

  match value.parse::<u64>() {
    Ok(count) => Some((name, count)),
    Err(error) => {
      diagnostics.push(Diagnostic::new(
        format!("Invalid `{key}` value `{value}`: {error}"),
        expression.text.range,
      ));

      None
    }
  }
}

fn keyword<'a>(
  attribute: &'a Attribute,
  key: &str,
) -> Option<(&'a TextNode, Option<&'a AttributeExpression>)> {
  match attribute.keyword_argument(key)? {
    AttributeArgument::Keyword { name, value, .. } => {
      Some((name, value.as_ref()))
    }
    AttributeArgument::Positional(_) => None,
  }
}

fn option(
  attribute: &Attribute,
  key: &str,
  parameter: &str,
) -> Option<(String, lsp::Range)> {
  match keyword(attribute, key)? {
    (_, Some(expression)) => {
      Some((expression.string_value()?, expression.text.range))
    }
    (name, None) => Some((parameter.to_owned(), name.range)),
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::{formatdoc, indoc},
  };

  #[test]
  fn arg_attribute_const_expression_kwargs() {
    AnalyzerTest::new(indoc! {
      "
      help := 'help'
      pattern := '[a-z]+'

      [arg('bar', help=help, pattern=pattern)]
      foo bar:
        echo {{ bar }}
      "
    })
    .run();
  }

  #[test]
  fn arg_attribute_duplicate_option() {
    #[track_caller]
    fn case(
      first: &str,
      second: &str,
      message: &'static str,
      start: u32,
      end: u32,
    ) {
      AnalyzerTest::new(&formatdoc! {
        "
        [arg('foo', {first})]
        [arg('bar', {second})]
        baz foo bar:
          echo {{{{foo}}}} {{{{bar}}}}
        "
      })
      .error(message, lsp::Range::at(1, start, 1, end))
      .run();
    }

    case(
      "long",
      "long='foo'",
      "Recipe `baz` defines option `--foo` multiple times",
      17,
      22,
    );

    case(
      "short='b'",
      "short",
      "Recipe `baz` defines option `-b` multiple times",
      12,
      17,
    );
  }

  #[test]
  fn arg_attribute_duplicate_parameter() {
    AnalyzerTest::new(indoc! {
      "
      [arg('foo', help='bar')]
      [arg('foo', long='foo')]
      bar foo:
        echo {{foo}}
      "
    })
    .error(
      "`[arg]` attribute for parameter `foo` is duplicated",
      lsp::Range::at(1, 0, 2, 0),
    )
    .run();
  }

  #[test]
  fn arg_attribute_invalid_options() {
    #[track_caller]
    fn case(
      arguments: &str,
      parameter: &str,
      message: &'static str,
      range: lsp::Range,
    ) {
      AnalyzerTest::new(&formatdoc! {
        "
        set unstable
        set lists

        [arg('foo', {arguments})]
        bar {parameter}:
          echo {{{{foo}}}}
        "
      })
      .error(message, range)
      .run();
    }

    case(
      "long='a=b'",
      "foo",
      "Option name for parameter `foo` contains equal sign",
      lsp::Range::at(3, 17, 3, 22),
    );

    case(
      "long='-a'",
      "foo",
      "Option name for parameter `foo` starts with dash",
      lsp::Range::at(3, 17, 3, 21),
    );

    case(
      "long=''",
      "foo",
      "Option name for parameter `foo` is empty",
      lsp::Range::at(3, 17, 3, 19),
    );

    case(
      "short='ab'",
      "foo",
      "Short option name for parameter `foo` contains multiple characters",
      lsp::Range::at(3, 18, 3, 22),
    );

    case(
      "long, flag='bar'",
      "foo",
      "`flag` attribute for argument `foo` takes no value",
      lsp::Range::at(3, 18, 3, 22),
    );

    case(
      "flag",
      "foo",
      "Argument attribute `flag` only valid with `long` or `short`",
      lsp::Range::at(3, 12, 3, 16),
    );

    case(
      "long, flag, value='bar'",
      "foo",
      "Argument `foo` may not have both `flag` and `value` attributes",
      lsp::Range::at(3, 18, 3, 22),
    );

    case(
      "long, flag, pattern='bar'",
      "foo",
      "Argument `foo` may not have both `flag` and `pattern` attributes",
      lsp::Range::at(3, 18, 3, 22),
    );

    case(
      "long, flag",
      "foo='bar'",
      "Flag parameter `foo` may not have a default",
      lsp::Range::at(4, 4, 4, 13),
    );

    case(
      "long, multiple='bar'",
      "foo",
      "Attribute key `multiple` takes no value",
      lsp::Range::at(3, 18, 3, 26),
    );

    case(
      "multiple",
      "foo",
      "Argument attribute `multiple` only valid with `long` or `short`",
      lsp::Range::at(3, 12, 3, 20),
    );

    case(
      "pattern='('",
      "foo",
      "Failed to parse argument pattern",
      lsp::Range::at(3, 12, 3, 19),
    );

    case(
      "pattern='(?x)#'",
      "foo",
      "Failed to parse argument pattern",
      lsp::Range::at(3, 12, 3, 19),
    );

    case(
      "min='01'",
      "+foo",
      "Invalid `min` value `01`",
      lsp::Range::at(3, 16, 3, 20),
    );

    case(
      "max='18446744073709551616'",
      "+foo",
      "Invalid `max` value `18446744073709551616`: number too large to fit in target type",
      lsp::Range::at(3, 16, 3, 38),
    );

    case(
      "min='1' + '2'",
      "+foo",
      "Attribute `arg` arguments must be string literals",
      lsp::Range::at(3, 16, 3, 25),
    );

    case(
      "min='2', max='1'",
      "+foo",
      "Argument attribute `min` `2` exceeds `max` `1`",
      lsp::Range::at(3, 12, 3, 15),
    );

    case(
      "min='1'",
      "foo",
      "Argument attribute `min` only valid with `multiple` or a variadic parameter",
      lsp::Range::at(3, 12, 3, 15),
    );
  }

  #[test]
  fn arg_attribute_unknown_parameter() {
    #[track_caller]
    fn case(name: &str, end: u32) {
      AnalyzerTest::new(&formatdoc! {
        "
        [arg({name}, help='bar')]
        foo baz:
          echo {{{{baz}}}}
        "
      })
      .error(
        "`[arg]` references unknown parameter `bar`",
        lsp::Range::at(0, 5, 0, end),
      )
      .run();
    }

    case("'bar'", 10);
    case("x'bar'", 11);
  }

  #[test]
  fn arg_attribute_valid() {
    #[track_caller]
    fn case(name: &str) {
      AnalyzerTest::new(&formatdoc! {
        "
        [arg({name}, help='bar')]
        bar foo:
          echo {{{{foo}}}}
        "
      })
      .run();
    }

    case("'foo'");
    case("'''foo'''");
    case(r#""\u{66}oo""#);
    case("x'foo'");
  }

  #[test]
  fn arg_attribute_value_accepts_expression() {
    AnalyzerTest::new(indoc! {
      "
      foo := 'foo'

      [arg('bar', long='bar', value=foo / 'baz')]
      bar bar:
        echo {{bar}}
      "
    })
    .run();
  }

  #[test]
  fn arg_attribute_value_requires_long_or_short() {
    AnalyzerTest::new(indoc! {
      "
      [arg('name', value='hi')]
      foo name:
        echo {{name}}
      "
    })
    .error(
      "Argument attribute `value` only valid with `long` or `short`",
      lsp::Range::at(0, 13, 0, 18),
    )
    .run();
  }

  #[test]
  fn arg_attribute_value_with_long_ok() {
    AnalyzerTest::new(indoc! {
      "
      [arg('name', long='name', value='hi')]
      foo name:
        echo {{name}}
      "
    })
    .run();
  }

  #[test]
  fn arg_attribute_with_flag() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      [arg('foo', long, flag)]
      bar foo:
        echo {{foo}}
      "
    })
    .run();
  }

  #[test]
  fn arg_attribute_with_long_option() {
    AnalyzerTest::new(indoc! {
      "
      [arg('foo', long=\"foo-opt\")]
      bar foo:
        echo {{foo}}
      "
    })
    .run();
  }

  #[test]
  fn arg_attribute_with_multiple_options() {
    AnalyzerTest::new(indoc! {
      "
      [arg('foo', long=\"foo-opt\", short=\"f\", value=\"default\")]
      bar foo:
        echo {{foo}}
      "
    })
    .run();
  }

  #[test]
  fn arg_attribute_with_pattern() {
    AnalyzerTest::new(indoc! {
      "
      [arg('version', pattern=\"[0-9]+\\\\.[0-9]+\\\\.[0-9]+\")]
      release version:
        echo {{version}}
      "
    })
    .run();
  }

  #[test]
  fn arg_attribute_with_short_option() {
    AnalyzerTest::new(indoc! {
      "
      [arg('foo', short=\"f\")]
      bar foo:
        echo {{foo}}
      "
    })
    .run();
  }

  #[test]
  fn arg_min_max_kwargs_accepted() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      [arg('FILES', min='2', max='4')]
      backup +FILES:
        scp {{FILES}} me@server.com:
      "
    })
    .run();
  }

  #[test]
  fn arg_multiple_kwarg_accepted() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      [arg('foo', long, multiple)]
      bar foo:
        echo {{foo}}
      "
    })
    .run();
  }
}
