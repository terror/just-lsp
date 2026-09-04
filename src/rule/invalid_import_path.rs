use super::*;

define_rule! {
  InvalidImportPathRule {
    id: "invalid-import-path",
    message: "invalid import path",
    run(context) {
      let document = context.document();

      let mut diagnostics = Vec::new();

      for import in document.imports() {
        if import.is_dynamic() {
          continue;
        }

        let path = match import.resolve(&document.uri) {
          Ok(Some(path)) => path,
          Ok(None) => continue,
          Err(Error::EmptyImportPath) if import.optional => continue,
          Err(error) => {
            diagnostics.push(Diagnostic::error(
              error.to_string(),
              import.path.range,
            ));

            continue;
          }
        };

        if !import.optional && !path.exists() {
          diagnostics.push(Diagnostic::error(
            format!("Import path does not exist: `{}`", path.display()),
            import.path.range,
          ));
        }
      }

      diagnostics
    }
  }
}
