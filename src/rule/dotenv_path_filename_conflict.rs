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
        .find(|setting| setting.name.value == "dotenv-filename");

      if dotenv_path && let Some(filename) = dotenv_filename {
        return vec![Diagnostic::warning(
          "`dotenv-path` overrides `dotenv-filename`".to_string(),
          filename.range,
        )];
      }

      vec![]
    }
  }
}
