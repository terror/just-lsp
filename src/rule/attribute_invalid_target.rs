use super::*;

define_rule! {
  /// Ensures attributes only appear on syntax nodes that actually accept
  /// attributes.
  AttributeInvalidTargetRule {
    id: "attribute-invalid-target",
    message: "invalid attribute target",
    run(ctx) {
      let mut diagnostics = Vec::new();

      for attribute in ctx.attributes() {
        let attribute_name = &attribute.name.value;

        if ctx.builtin_attributes(attribute_name).is_empty() {
          continue;
        }

        if attribute.target.is_none() {
          diagnostics.push(Diagnostic::error(
            format!("Attribute `{attribute_name}` applied to invalid target",),
            attribute.range,
          ));
        }
      }

      diagnostics
    }
  }
}
