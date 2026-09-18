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

          if !recipe.parameters.iter().any(|parameter| parameter.name == parameter_name) {
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
