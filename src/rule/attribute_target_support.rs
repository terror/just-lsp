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

        let matching = context.builtin_attributes(attribute_name);

        if matching.is_empty() {
          continue;
        }

        let Some(target_type) = attribute.target else {
          continue;
        };

        let is_valid_target = matching.iter().copied().any(|attr| {
          if let Builtin::Attribute { targets, .. } = attr {
            targets.contains(&target_type)
          } else {
            false
          }
        });

        if !is_valid_target {
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
