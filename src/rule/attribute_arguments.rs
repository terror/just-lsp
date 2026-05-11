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

        let ranges = matching
          .iter()
          .copied()
          .filter_map(|attribute| match attribute {
            Builtin::Attribute { kind, .. } => Some(kind.argument_range()),
            _ => None,
          })
          .collect::<Vec<_>>();

        let is_valid = ranges
          .iter()
          .any(|range| range.contains(&argument_count));

        if is_valid {
          continue;
        }

        let min = ranges.iter().map(|range| *range.start()).min().unwrap_or(0);

        let max = ranges
          .iter()
          .map(|range| *range.end())
          .try_fold(0, |acc, max| {
            if max == usize::MAX { None } else { Some(acc.max(max)) }
          });

        let expected = match max {
          Some(max) if min == max => format!("{min}"),
          Some(max) => format!("{min}-{max}"),
          None => format!("at least {min}"),
        };

        diagnostics.push(Diagnostic::error(
          format!(
            "Attribute `{attribute_name}` got {argument_count} {} but takes {expected} {}",
            Count("argument", argument_count),
            if min == 1 && max.is_none_or(|max| max == 1) { "argument" } else { "arguments" },
          ),
          attribute.range,
        ));
      }

      diagnostics
    }
  }
}
