use super::*;

define_rule! {
  /// Validates that each attribute is attached to a supported target kind
  /// (recipe, module, alias, etc.) according to the builtin metadata.
  AttributeTargetSupportRule {
    id: "attribute-target-support",
    message: "unsupported attribute target",
    run(context) {
      let mut diagnostics = Vec::new();

      for attribute in context.attributes() {
        let attribute_name = &attribute.name.value;

        let Some(Builtin::Attribute { targets, .. }) =
          context.builtin_attribute(attribute_name)
        else {
          continue;
        };

        let Some(target_type) = attribute.target else {
          continue;
        };

        if !targets.contains(&target_type) {
          diagnostics.push(Diagnostic::error(
            format!(
              "Attribute `{attribute_name}` cannot be applied to {target_type} target",
            ),
            attribute.range,
          ));
        }
      }

      diagnostics
    }
  }
}
