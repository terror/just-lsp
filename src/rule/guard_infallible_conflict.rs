use super::*;

define_rule! {
  GuardInfallibleConflictRule {
    id: "guard-infallible-conflict",
    title: "guard and infallible sigils",
    severity: Error,
  }
}

impl GuardInfallibleConflictRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    if !context.setting_enabled("guards") {
      return Vec::new();
    }

    let default_script = context.setting_enabled("default-script");

    context
      .local_declarations(context.recipes())
      .filter(|recipe| !recipe.runs_as_script(default_script))
      .flat_map(|recipe| {
        recipe
          .body_lines()
          .scan(false, |continued, line| {
            Some((mem::replace(continued, line.continues), line))
          })
          .filter(|(continued, _)| !continued)
          .filter_map(|(_, line)| {
            let length = line
              .text
              .chars()
              .scan(String::new(), |seen, c| {
                (matches!(c, '-' | '?' | '@') && !seen.contains(c)).then(|| {
                  seen.push(c);
                })
              })
              .count();

            let sigils = &line.text[..length];

            (sigils.contains('?') && sigils.contains('-')).then(|| {
              let start = u32::try_from(line.indent.len()).unwrap_or(u32::MAX);

              Diagnostic::new(
                "the guard `?` and infallible `-` sigils may not be used together",
                lsp::Range::at(
                  line.number,
                  start,
                  line.number,
                  start.saturating_add(u32::try_from(length).unwrap_or(u32::MAX)),
                ),
              )
            })
          })
          .collect::<Vec<_>>()
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn guard_infallible_conflict() {
    AnalyzerTest::new(indoc! {
      "
      set guards

      foo:
        ?-bar
        -@?bar \\
        ?-baz
        --?bar
      "
    })
    .error(
      "the guard `?` and infallible `-` sigils may not be used together",
      lsp::Range::at(3, 2, 3, 4),
    )
    .error(
      "the guard `?` and infallible `-` sigils may not be used together",
      lsp::Range::at(4, 2, 4, 5),
    )
    .run();

    AnalyzerTest::new(indoc! {
      "
      foo:
        ?-bar
      "
    })
    .run();

    AnalyzerTest::new(indoc! {
      "
      set guards

      foo:
        #!/bin/sh
        ?-bar
      "
    })
    .run();
  }
}
