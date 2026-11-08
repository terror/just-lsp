use super::*;

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
        let max = Self::count(attribute, "max", &mut diagnostics);
        let min = Self::count(attribute, "min", &mut diagnostics);

        if let (Some((min, range)), Some((max, _))) = (min, max)
          && min > max
        {
          diagnostics.push(Diagnostic::error(
            format!("`[arg]` `min` `{min}` exceeds `max` `{max}`"),
            range,
          ));
        }

        if let Some(value) = attribute.keyword_argument("value")
          && attribute.keyword_argument("long").is_none()
          && attribute.keyword_argument("short").is_none()
        {
          diagnostics.push(Diagnostic::error(
            "`[arg]` `value=` requires `long=` or `short=`",
            value.range(),
          ));
        }
      }

      for recipe in context.local_declarations(context.recipes()) {
        let mut seen = HashSet::new();

        for attribute in recipe
          .attributes
          .iter()
          .filter(|attribute| attribute.name.value == "arg")
        {
          let Some(name) = attribute.positional_arguments().next() else {
            continue;
          };

          if name.kind != AttributeExpressionKind::StringLiteral {
            continue;
          }

          let Some(parameter_name) = StringLiteral::parse(&name.text.value)
            .ok()
            .flatten()
            .filter(|literal| !literal.shell_expanded)
            .map(|literal| literal.cooked)
          else {
            continue;
          };

          if let Some(parameter) = recipe
            .parameters
            .iter()
            .find(|parameter| parameter.name == parameter_name)
          {
            if !matches!(parameter.kind, ParameterKind::Variadic(_))
              && attribute.keyword_argument("multiple").is_none()
              && let Some(AttributeArgument::Keyword { name, .. }) = attribute
                .keyword_argument("max")
                .or_else(|| attribute.keyword_argument("min"))
            {
              diagnostics.push(Diagnostic::error(
                format!(
                  "`[arg]` `{}` requires `multiple` or a variadic parameter",
                  name.value,
                ),
                name.range,
              ));
            }
          } else {
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
        }
      }

      diagnostics
    }
  }
}

impl InvalidArgAttributeRule {
  fn count(
    attribute: &Attribute,
    key: &str,
    diagnostics: &mut Vec<Diagnostic>,
  ) -> Option<(u64, lsp::Range)> {
    let AttributeArgument::Keyword {
      name,
      value: Some(value),
      ..
    } = attribute.keyword_argument(key)?
    else {
      return None;
    };

    if value.kind != AttributeExpressionKind::StringLiteral {
      return None;
    }

    let literal = StringLiteral::parse(&value.text.value)
      .ok()
      .flatten()
      .filter(|literal| !literal.shell_expanded)?;

    let cooked = literal.cooked;

    if cooked.is_empty()
      || cooked.starts_with('0') && cooked.len() > 1
      || !cooked.bytes().all(|byte| byte.is_ascii_digit())
    {
      diagnostics.push(Diagnostic::error(
        format!("Invalid `{key}` value `{cooked}`"),
        value.text.range,
      ));
      return None;
    }

    match cooked.parse::<u64>() {
      Ok(count) => Some((count, name.range)),
      Err(error) => {
        diagnostics.push(Diagnostic::error(
          format!("Invalid `{key}` value `{cooked}`: {error}"),
          value.text.range,
        ));
        None
      }
    }
  }
}
