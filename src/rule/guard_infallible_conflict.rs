use super::*;

define_rule! {
  GuardInfallibleConflictRule {
    id: "guard-infallible-conflict",
    message: "guard and infallible sigils",
    run(context) {
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

                Diagnostic::error(
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
}
