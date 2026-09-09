use super::*;

define_rule! {
  DotenvPathFilenameConflictRule {
    id: "dotenv-path-filename-conflict",
    message: "conflicting dotenv settings",
    run(context) {
      let dotenv_path = context
        .settings()
        .iter()
        .any(|setting| setting.name.value == "dotenv-path");

      let dotenv_filename = context
        .settings()
        .iter()
        .any(|setting| setting.name.value == "dotenv-filename");

      context
        .local_declarations(context.settings())
        .find(|setting| setting.name.value == "dotenv-filename" && dotenv_path)
        .or_else(|| {
          context
            .local_declarations(context.settings())
            .find(|setting| setting.name.value == "dotenv-path" && dotenv_filename)
        })
        .into_iter()
        .map(|setting| {
          Diagnostic::warning(
            "`dotenv-path` overrides `dotenv-filename`".to_string(),
            setting.range,
          )
        })
        .collect()
    }
  }
}
