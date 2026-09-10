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
        .filter(|recipe| !recipe.runs_as_script(default_script))
        .filter_map(|recipe| {
          recipe
            .body_lines()
            .try_fold(None, |expected_kind: Option<IndentKind>, line| {
              if line.kind == IndentKind::Mixed
                || expected_kind.is_some_and(|expected| expected != line.kind)
              {
                ControlFlow::Break(Diagnostic::error(
                  format!(
                    "Recipe `{}` mixes tabs and spaces for indentation",
                    recipe.name.value,
                  ),
                  line.range(),
                ))
              } else {
                ControlFlow::Continue(Some(line.kind))
              }
            })
            .break_value()
        })
        .collect()
    }
  }
}
