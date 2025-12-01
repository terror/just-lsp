use super::*;

define_rule! {
  /// Warns when an attribute name isn't part of the known builtin attribute set.
  UnknownAttributeRule {
    id: "unknown-attribute",
    message: "unknown attribute",
    run(context) {
      let mut diagnostics = Vec::new();

      for attribute in context.attributes() {
        let attribute_name = &attribute.name.value;

        if context.builtin_attributes(attribute_name).is_empty() {
          diagnostics.push(Diagnostic::error(
            format!("Unknown attribute `{attribute_name}`"),
            attribute.name.range,
          ));
        }
      }

      diagnostics
    }
  }
}
