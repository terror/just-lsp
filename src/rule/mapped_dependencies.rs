use super::*;

define_rule! {
  MappedDependenciesRule {
    id: "mapped-dependencies",
    message: "invalid mapped dependency",
    run(context) {
      let mut diagnostics = Vec::new();
      let lists = context.setting_enabled("lists");

      for recipe in context.recipes() {
        for dependency in &recipe.dependencies {
          let starred = dependency
            .arguments
            .iter()
            .filter_map(|argument| argument.starred)
            .collect::<Vec<_>>();

          match dependency.mapped {
            Some(range) => {
              if !lists {
                diagnostics.push(Diagnostic::error(
                  "Mapped dependencies require `set lists`".to_string(),
                  range,
                ));
              }

              if starred.is_empty() {
                diagnostics.push(Diagnostic::error(
                  "Mapped dependencies must include a starred argument".to_string(),
                  range,
                ));
              }

              for range in starred.iter().skip(1) {
                diagnostics.push(Diagnostic::error(
                  "Mapped dependencies may not include multiple starred arguments".to_string(),
                  *range,
                ));
              }
            }
            None => {
              for range in starred {
                diagnostics.push(Diagnostic::error(
                  "Starred dependency arguments require mapped dependencies".to_string(),
                  range,
                ));
              }
            }
          }
        }
      }

      diagnostics
    }
  }
}
