use super::*;

/// Warns when an attribute name isn’t part of the known builtin attribute set.
pub(crate) struct UnknownAttributeRule;

impl Rule for UnknownAttributeRule {
  fn id(&self) -> &'static str {
    "unknown-attribute"
  }

  fn message(&self) -> &'static str {
    "unknown attribute"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<Diagnostic> {
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
