use super::*;

define_rule! {
  InvalidImportPathRule {
    id: "invalid-import-path",
    message: "invalid import path",
    run(context) {
      let document = context.document();

      document
        .imports()
        .into_iter()
        .filter_map(|import| {
          let message = match import.resolve(&document.uri) {
            Ok(Some(path)) if !import.optional && !path.exists() => {
              format!("Import path does not exist: `{}`", path.display())
            }
            Err(Error::EmptyImportPath) if import.optional => return None,
            Err(error) => error.to_string(),
            _ => return None,
          };

          Some(Diagnostic::error(message, import.path.range))
        })
        .collect()
    }
  }
}
