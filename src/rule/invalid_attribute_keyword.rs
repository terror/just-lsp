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

        for argument in &attribute.arguments {
          let AttributeArgument::Keyword { name, value, range } = argument
          else {
            continue;
          };

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
