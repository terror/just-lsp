use super::*;

define_rule! {
  InvalidRecipeParameterOrderRule {
    id: "invalid-recipe-parameter-order",
    message: "invalid recipe parameter order",
    run(context) {
      let mut diagnostics = Vec::new();

      for recipe in context.local_declarations(context.recipes()) {
        let attributes = recipe
          .attributes
          .iter()
          .filter(|attribute| attribute.name.value == "arg")
          .filter_map(|attribute| {
            Some((
              Self::literal(attribute.positional_arguments().next()?)?,
              attribute,
            ))
          })
          .collect::<HashMap<_, _>>();

        let mut passed_default = false;

        for param in &recipe.parameters {
          let attribute = attributes.get(&param.name);

          if attribute.is_some_and(|attribute| {
            attribute.keyword_argument("long").is_some()
              || attribute.keyword_argument("short").is_some()
          }) {
            continue;
          }

          let min = attribute.and_then(|attribute| {
            let AttributeArgument::Keyword { value: Some(value), .. } =
              attribute.keyword_argument("min")?
            else {
              return None;
            };

            Self::literal(value)?.parse::<u64>().ok()
          });

          let has_default = param.default_value.is_some();

          if passed_default
            && !has_default
            && (!matches!(
              param.kind,
              ParameterKind::Variadic(VariadicType::ZeroOrMore)
            ) || min.is_some_and(|min| min > 0))
          {
            diagnostics.push(Diagnostic::error(
              format!(
                "Required parameter `{}` follows a parameter with a default value",
                param.name
              ),
              param.range,
            ));
          }

          if has_default {
            passed_default = true;
          }
        }
      }

      diagnostics
    }
  }
}

impl InvalidRecipeParameterOrderRule {
  fn literal(expression: &AttributeExpression) -> Option<String> {
    if expression.kind != AttributeExpressionKind::StringLiteral {
      return None;
    }

    let literal = StringLiteral::parse(&expression.text.value).ok()??;

    if literal.shell_expanded {
      Some(
        shellexpand::full_with_context(
          &literal.cooked,
          || env::home_dir()?.into_os_string().into_string().ok(),
          |name| env::var(name).map(Some),
        )
        .ok()?
        .into_owned(),
      )
    } else {
      Some(literal.cooked)
    }
  }
}
