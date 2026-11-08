use super::*;

define_rule! {
  InvalidArgOptionRule {
    id: "invalid-arg-option",
    message: "invalid argument option",
    run(context) {
      let mut diagnostics = Vec::new();

      for recipe in context.local_declarations(context.recipes()) {
        let mut longs = HashSet::new();
        let mut shorts = HashSet::new();

        for attribute in recipe
          .attributes
          .iter()
          .filter(|attribute| attribute.name.value == "arg")
        {
          let Some(parameter) = attribute
            .positional_arguments()
            .next()
            .and_then(Self::literal)
          else {
            continue;
          };

          for (keyword, prefix, seen) in [
            ("long", "--", &mut longs),
            ("short", "-", &mut shorts),
          ] {
            let Some(AttributeArgument::Keyword { name, value, .. }) =
              attribute.keyword_argument(keyword)
            else {
              continue;
            };

            let (option, range) = if let Some(value) = value {
              let Some(option) = Self::literal(value) else {
                continue;
              };

              let error = if option.contains('=') {
                Some(format!(
                  "option name for parameter `{parameter}` contains equal sign"
                ))
              } else if option.starts_with('-') {
                Some(format!(
                  "option name for parameter `{parameter}` starts with dash"
                ))
              } else if option.is_empty() {
                Some(format!(
                  "option name for parameter `{parameter}` is empty"
                ))
              } else if keyword == "short" && option.chars().count() != 1 {
                Some(format!(
                  "short option name for parameter `{parameter}` contains multiple characters"
                ))
              } else {
                None
              };

              if let Some(error) = error {
                diagnostics.push(Diagnostic::error(error, value.text.range));
                continue;
              }

              (option, value.text.range)
            } else {
              (parameter.clone(), name.range)
            };

            let option = if keyword == "short" {
              let Some(option) = option.chars().next() else {
                continue;
              };

              option.to_string()
            } else {
              option
            };

            if !seen.insert(option.clone()) {
              diagnostics.push(Diagnostic::error(
                format!(
                  "recipe `{}` defines option `{prefix}{option}` multiple times",
                  recipe.name.value,
                ),
                range,
              ));
            }
          }
        }
      }

      diagnostics
    }
  }
}

impl InvalidArgOptionRule {
  fn literal(expression: &AttributeExpression) -> Option<String> {
    if expression.kind != AttributeExpressionKind::StringLiteral {
      return None;
    }

    StringLiteral::parse(&expression.text.value)
      .ok()
      .flatten()
      .filter(|literal| !literal.shell_expanded)
      .map(|literal| literal.cooked)
  }
}
