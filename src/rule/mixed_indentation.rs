use super::*;

define_rule! {
  /// Detects recipes that mix tabs and spaces for indentation, which often
  /// results in confusing or invalid `just` bodies.
  MixedIndentationRule {
    id: "mixed-recipe-indentation",
    message: "mixed indentation",
    run(context) {
      let default_script = context.setting_enabled("default-script");

      context
        .local_declarations(context.recipes())
        .filter_map(|recipe| {
          let script = recipe.runs_as_script(default_script);

          recipe
            .body_lines()
            .try_fold(None, |expected: Option<RecipeLine>, line| {
              let mixed = match &expected {
                None => line.kind == IndentKind::Mixed,
                Some(expected) if script => {
                  !line.indent.starts_with(expected.indent)
                    && line.mixed_prefix(expected.indent.len())
                }
                Some(expected) => {
                  line.kind == IndentKind::Mixed || expected.kind != line.kind
                }
              };

              if mixed {
                ControlFlow::Break(Diagnostic::error(
                  format!(
                    "Recipe `{}` mixes tabs and spaces for indentation",
                    recipe.name.value,
                  ),
                  line.range(),
                ))
              } else {
                ControlFlow::Continue(Some(expected.unwrap_or(line)))
              }
            })
            .break_value()
        })
        .collect()
    }
  }
}
