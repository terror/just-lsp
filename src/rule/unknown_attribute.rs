use super::*;

define_rule! {
  /// Warns when an attribute name isn't part of the known builtin attribute set.
  UnknownAttributeRule {
    id: "unknown-attribute",
    message: "unknown attribute",
    provides_quickfixes: true,
    run(context) {
      let mut diagnostics = Vec::new();

      for attribute in context.attributes() {
        let attribute_name = &attribute.name.value;

        if context.builtin_attributes(attribute_name).is_empty() {
          let suggestion = attribute_name.find_suggestion(
            BUILTINS.iter().filter_map(|builtin| match builtin {
              Builtin::Attribute { name, .. } => Some(*name),
              _ => None,
            }),
          );

          let message = match &suggestion {
            Some(suggestion) => format!(
              "Unknown attribute `{attribute_name}`. Did you mean `{suggestion}`?"
            ),
            None => format!("Unknown attribute `{attribute_name}`"),
          };

          let quickfix = suggestion.map(|suggestion| {
            Quickfix::replacement(&attribute.name, suggestion)
          });

          diagnostics.push(
            Diagnostic::error(message, attribute.name.range)
              .quickfix(quickfix),
          );
        }
      }

      diagnostics
    }
  }
}
