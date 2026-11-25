use super::*;

define_rule! {
  InvalidImportPathRule {
    id: "invalid-import-path",
    title: "invalid import path",
    severity: Error,
  }
}

impl InvalidImportPathRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let document = context.document();

    let view = context.view();

    document
      .imports()
      .into_iter()
      .filter_map(|import| {
        if import.is_dynamic() {
          return Some(Diagnostic::new(
            "Expected keyword `x` but found identifier `f`",
            import.path.range,
          ));
        }

        let message = match import.resolve(&document.uri) {
          Ok(Some(path))
            if import.is_enabled()
              && !import.optional
              && !view.loads(&path) =>
          {
            format!("Import path does not exist: `{}`", path.display())
          }
          Err(Error::EmptyImportPath)
            if import.optional || !import.is_enabled() =>
          {
            return None;
          }
          Err(Error::InvalidStringEscape) => return None,
          Err(error) => error.to_string(),
          _ => return None,
        };

        Some(Diagnostic::new(message, import.path.range))
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn import_directory_path() {
    let expected = if cfg!(windows) {
      "Import path does not exist: `C:\\.`"
    } else {
      "Import path does not exist: `/.`"
    };

    AnalyzerTest::new("import '.'")
      .error(expected, lsp::Range::at(0, 7, 0, 10))
      .run();
  }

  #[test]
  fn import_disabled_paths_are_ignored() {
    #[track_caller]
    fn case(path: &str) {
      let attribute = if cfg!(windows) { "unix" } else { "windows" };

      AnalyzerTest::new(&format!("[{attribute}]\nimport {path}\n")).run();
    }

    case("'foo.just'");
    case("x'foo.just'");
    case("''");
    case("x''");
  }

  #[test]
  fn import_format_string() {
    AnalyzerTest::new("import f'foo'")
      .error(
        "Expected keyword `x` but found identifier `f`",
        lsp::Range::at(0, 7, 0, 13),
      )
      .run();
  }

  #[test]
  fn import_invalid_path() {
    let expected = if cfg!(windows) {
      "Import path does not exist: `C:\\foo?bar.just`"
    } else {
      "Import path does not exist: `/foo?bar.just`"
    };

    AnalyzerTest::new(indoc! {
      "
      import 'foo?bar.just'
      "
    })
    .error(expected, lsp::Range::at(0, 7, 0, 21))
    .run();
  }

  #[test]
  fn import_optional_invalid_path() {
    AnalyzerTest::new(indoc! {
      "
      import? 'foo?bar.just'
      "
    })
    .run();
  }

  #[test]
  fn import_paths_use_filesystem() {
    AnalyzerTest::new("import 'foo.just'")
      .file("foo.just", "")
      .run();

    let expected = if cfg!(windows) {
      "Import path does not exist: `C:\\foo.just`"
    } else {
      "Import path does not exist: `/foo.just`"
    };

    AnalyzerTest::new("import 'foo.just'")
      .error(expected, lsp::Range::at(0, 7, 0, 17))
      .run();

    let expected = if cfg!(windows) {
      "Import path does not exist: `C:\\foo`"
    } else {
      "Import path does not exist: `/foo`"
    };

    AnalyzerTest::new("import 'foo'")
      .file("foo/bar.just", "")
      .error(expected, lsp::Range::at(0, 7, 0, 12))
      .run();
  }

  #[test]
  fn import_shell_expanded_string_path_is_checked() {
    let expected = if cfg!(windows) {
      "Import path does not exist: `C:\\nonexistent.just`"
    } else {
      "Import path does not exist: `/nonexistent.just`"
    };

    AnalyzerTest::new(indoc! {
      "
      import x'nonexistent.just'
      "
    })
    .error(expected, lsp::Range::at(0, 7, 0, 26))
    .run();
  }
}
