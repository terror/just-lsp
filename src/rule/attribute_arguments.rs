use super::*;

define_rule! {
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

        let bounds = matching
          .iter()
          .copied()
          .filter_map(|attr| match attr {
            Builtin::Attribute { min_args, max_args, .. } => Some((*min_args, *max_args)),
            _ => None,
          })
          .collect::<Vec<_>>();

        let is_valid = bounds
          .iter()
          .any(|(min, max)| argument_count >= *min && max.is_none_or(|m| argument_count <= m));

        if is_valid {
          continue;
        }

        let min = bounds.iter().map(|(min, _)| *min).min().unwrap_or(0);

        let max = bounds
          .iter()
          .map(|(_, max)| *max)
          .try_fold(0, |acc, max| max.map(|value| acc.max(value)));

        let expected = match max {
          Some(max) if min == max => format!("{min}"),
          Some(max) => format!("{min}-{max}"),
          None => format!("at least {min}"),
        };

        diagnostics.push(Diagnostic::error(
          format!(
            "Attribute `{attribute_name}` got {argument_count} {} but takes {expected} {}",
            Count("argument", argument_count),
            if min == 1 && max.is_none_or(|m| m == 1) { "argument" } else { "arguments" },
          ),
          attribute.range,
        ));
      }

      diagnostics
    }
  }
}
