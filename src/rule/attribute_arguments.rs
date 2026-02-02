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

        let matching = context.builtin_attributes(attribute_name);

        if matching.is_empty() {
          continue;
        }

        let argument_count = attribute.arguments.len();
        let has_arguments = argument_count > 0;

        if attribute_name == "arg" {
          if argument_count == 0 {
            diagnostics.push(Diagnostic::error(
              "Attribute `arg` requires at least 1 argument (parameter name)".to_string(),
              attribute.range,
            ));
          }
          continue;
        }

        let parameter_mismatch = matching.iter().copied().all(|attr| {
          if let Builtin::Attribute { parameters, .. } = attr {
            (parameters.is_some() && !has_arguments)
              || (parameters.is_none() && has_arguments)
              || (parameters.map_or(0, |_| 1) < argument_count)
          } else {
            false
          }
        });

        if parameter_mismatch {
          let required_argument_count = matching
            .iter()
            .copied()
            .find_map(|attr| {
              if let Builtin::Attribute { parameters, .. } = attr {
                parameters.map(|_| 1)
              } else {
                None
              }
            })
            .unwrap_or(0);

          diagnostics.push(Diagnostic::error(
            format!(
              "Attribute `{attribute_name}` got {argument_count} {} but takes {required_argument_count} {}",
              Count("argument", argument_count),
              Count("argument", required_argument_count),
            ),
            attribute.range,
          ));
        }
      }

      diagnostics
    }
  }
}
