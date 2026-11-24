use super::*;

static COUNT: LazyLock<Regex> =
  LazyLock::new(|| Regex::new("^(0|[1-9][0-9]*)$").unwrap());

define_rule! {
  /// Validates `[arg(NAME, ...)]` attributes: that NAME refers to an existing
  /// recipe parameter, that only known keyword arguments are used, and that
  /// `value=` is paired with `long=` or `short=`.
  InvalidArgAttributeRule {
    id: "invalid-arg-attribute",
    message: "invalid arg attribute",
    run(context) {
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
            diagnostics.push(Diagnostic::error(message, expression.text.range));
          }
        }

        let has_option = long.is_some() || short.is_some();

        let pattern = keyword(attribute, "pattern");
        let value = keyword(attribute, "value");

        if let Some((key, _)) = value
          && !has_option
        {
          diagnostics.push(Diagnostic::error(
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
            diagnostics.push(Diagnostic::error(message, key.range));
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
            diagnostics.push(Diagnostic::error(message, key.range));
          }
        }

        if let Some((key, Some(expression))) = pattern
          && let Some(pattern) = expression.string_value()
          && (Regex::new(&pattern).is_err()
            || Regex::new(&format!("^(?:{pattern})$")).is_err())
        {
          diagnostics.push(Diagnostic::error(
            "Failed to parse argument pattern",
            key.range,
          ));
        }

        let max = count(attribute, "max", &mut diagnostics);
        let min = count(attribute, "min", &mut diagnostics);

        if let (Some((_, max)), Some((key, min))) = (max, min)
          && min > max
        {
          diagnostics.push(Diagnostic::error(
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
            diagnostics.push(Diagnostic::error(
              format!("`[arg]` references unknown parameter `{parameter_name}`"),
              name.text.range,
            ));
          }

          if !seen.insert(parameter_name.clone()) {
            diagnostics.push(Diagnostic::error(
              format!(
                "`[arg]` attribute for parameter `{parameter_name}` is duplicated"
              ),
              attribute.range,
            ));
          }

          if let Some((long, range)) = option(attribute, "long", &parameter_name)
            && !longs.insert(long.clone())
          {
            diagnostics.push(Diagnostic::error(
              format!(
                "Recipe `{}` defines option `--{long}` multiple times",
                recipe.name.value
              ),
              range,
            ));
          }

          if let Some((short, range)) = option(attribute, "short", &parameter_name)
            && let Some(short) = short.chars().next()
            && !shorts.insert(short)
          {
            diagnostics.push(Diagnostic::error(
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
            diagnostics.push(Diagnostic::error(
              format!("Flag parameter `{parameter_name}` may not have a default"),
              parameter.range,
            ));
          }

          if let Some((key, _)) =
            keyword(attribute, "max").or_else(|| keyword(attribute, "min"))
            && keyword(attribute, "multiple").is_none()
            && parameter.kind == ParameterKind::Normal
          {
            diagnostics.push(Diagnostic::error(
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
    diagnostics.push(Diagnostic::error(
      format!("Invalid `{key}` value `{value}`"),
      expression.text.range,
    ));

    return None;
  }

  match value.parse::<u64>() {
    Ok(count) => Some((name, count)),
    Err(error) => {
      diagnostics.push(Diagnostic::error(
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
