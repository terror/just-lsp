use super::*;

define_rule! {
  /// Reports attribute invocations whose argument counts don't match their
  /// builtin definitions.
  AttributeArgumentsRule {
    id: "attribute-arguments",
    message: "invalid attribute arguments",
    run(context) {
      let mut diagnostics = Vec::new();

      for attribute in context.attributes() {
        let attribute_name = &attribute.name.value;

        let Some(Builtin::Attribute { kind, .. }) =
          context.builtin_attribute(attribute_name)
        else {
          continue;
        };

        let argument_count = attribute.arguments.len();

        let range = kind.argument_range();

        if range.contains(&argument_count) {
          continue;
        }

        let (min, max) = (*range.start(), *range.end());

        let expected = match max {
          usize::MAX => format!("at least {min}"),
          _ if min == max => format!("{min}"),
          _ => format!("{min}-{max}"),
        };

        diagnostics.push(Diagnostic::error(
          format!(
            "Attribute `{attribute_name}` got {argument_count} {} but takes {expected} {}",
            Count("argument", argument_count),
            if min == 1 && matches!(max, 1 | usize::MAX) { "argument" } else { "arguments" },
          ),
          attribute.range,
        ));
      }

      diagnostics
    }
  }
}
