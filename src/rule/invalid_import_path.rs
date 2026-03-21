use super::*;

define_rule! {
  InvalidImportPathRule {
    id: "invalid-import-path",
    message: "invalid import path",
    run(context) {
      let document = context.document();

      let mut diagnostics = Vec::new();

      for import in document.imports() {
        if import.optional {
          continue;
        }

        let raw = &import.path.value;

        if raw.starts_with('f') || raw.starts_with('x') {
          continue;
        }

        let Some(path) = import.resolve(&document.uri) else {
          continue;
        };

        if !path.exists() {
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
