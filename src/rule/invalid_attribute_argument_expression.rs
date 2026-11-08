use super::*;

define_rule! {
  InvalidAttributeArgumentExpressionRule {
    id: "invalid-attribute-argument-expression",
    message: "invalid attribute argument expression",
    run(context) {
      let mut diagnostics = Vec::new();
      let mut expressions = ConstExpressions::new(context.view());

      for attribute in context.attributes() {
        let attribute_name = &attribute.name.value;

        let Some(Builtin::Attribute { signature, .. }) =
          context.builtin_attribute(attribute_name)
        else {
          continue;
        };

        for argument in &attribute.arguments {
          let (kind, expression) = match argument {
            AttributeArgument::Positional(expression) => {
              (signature.expression, expression)
            }
            AttributeArgument::Keyword { name, value: Some(value), .. } => {
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

          let expected = match kind {
            AttributeExpressionKind::Const
              if expressions.is_non_const(&expression.text) => {
              "const expressions"
            }
            AttributeExpressionKind::StringLiteral
              if expression.kind != AttributeExpressionKind::StringLiteral => {
              "string literals"
            }
            _ => continue,
          };

          diagnostics.push(Diagnostic::error(
            format!("Attribute `{attribute_name}` arguments must be {expected}"),
            expression.text.range,
          ));
        }
      }

      diagnostics
    }
  }
}
