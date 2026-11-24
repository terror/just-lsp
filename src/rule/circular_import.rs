use super::*;

define_rule! {
  CircularImportRule {
    id: "circular-import",
    message: "circular import",
    run(context) {
      let Ok(current) = context.document().uri.file_path() else {
        return Vec::new();
      };

      context
        .view()
        .circular_dependencies()
        .iter()
        .filter_map(|dependency| {
          let ProjectDependencyTarget::Path(target) = &dependency.target else {
            return None;
          };

          Some(Diagnostic::error(
            format!(
              "Import `{}` in `{}` is circular",
              target.file_path().ok()?.display(),
              current.display(),
            ),
            dependency.location,
          ))
        })
        .collect()
    }
  }
}
