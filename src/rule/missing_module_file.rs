use super::*;

define_rule! {
  MissingModuleFileRule {
    id: "missing-module-file",
    message: "missing module file",
    run(context) {
      context
        .view()
        .dependencies()
        .filter_map(|dependency| {
          Some(Diagnostic::error(
            format!(
              "Could not find source file for module `{}`",
              dependency.missing_module()?
            ),
            dependency.location,
          ))
        })
        .collect()
    }
  }
}
