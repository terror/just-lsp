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

        let is_valid = matching.iter().copied().any(|attr| {
          if let Builtin::Attribute { min_args, max_args, .. } = attr {
            argument_count >= *min_args
              && max_args.is_none_or(|max| argument_count <= max)
          } else {
            false
          }
        });

        if !is_valid {
          let (min, max) = matching
            .iter()
            .copied()
            .filter_map(|attr| {
              if let Builtin::Attribute { min_args, max_args, .. } = attr {
                Some((*min_args, *max_args))
              } else {
                None
              }
            })
            .fold((usize::MAX, Some(0)), |(acc_min, acc_max), (min, max)| {
              (
                acc_min.min(min),
                match (acc_max, max) {
                  (Some(a), Some(b)) => Some(a.max(b)),
                  _ => None,
                },
              )
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
              if max == Some(1) && min == 1 { "argument" } else { "arguments" },
            ),
            attribute.range,
          ));
        }
      }

      diagnostics
    }
  }
}
