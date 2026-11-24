use super::*;

define_rule! {
  InvalidImportPathRule {
    id: "invalid-import-path",
    message: "invalid import path",
    run(context) {
      let document = context.document();

      let view = context.view();

      document
        .imports()
        .into_iter()
        .filter_map(|import| {
          if import.is_dynamic() {
            return Some(Diagnostic::error(
              "Expected keyword `x` but found identifier `f`",
              import.path.range,
            ));
          }

          let message = match import.resolve(&document.uri) {
            Ok(Some(path))
              if import.is_enabled() && !import.optional && !view.loads(&path) =>
            {
              format!("Import path does not exist: `{}`", path.display())
            }
            Err(Error::EmptyImportPath)
              if import.optional || !import.is_enabled() => return None,
            Err(Error::InvalidStringEscape) => return None,
            Err(error) => error.to_string(),
            _ => return None,
          };

          Some(Diagnostic::error(message, import.path.range))
        })
        .collect()
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

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
    #[track_caller]
    fn case(files: &[(&str, &str)], expected: &[Diagnostic]) {
      assert_eq!(
        Analyzer {
          config: None,
          view: TestModule::new(files).view(),
        }
        .analyze(),
        expected,
      );
    }

    case(&[("justfile", "import 'foo.just'"), ("foo.just", "")], &[]);

    let path = if cfg!(windows) {
      "C:\\foo.just"
    } else {
      "/foo.just"
    };

    case(
      &[("justfile", "import 'foo.just'")],
      &[Diagnostic {
        display: "invalid import path".into(),
        id: "invalid-import-path".into(),
        ..Diagnostic::error(
          format!("Import path does not exist: `{path}`"),
          lsp::Range::at(0, 7, 0, 17),
        )
      }],
    );

    let path = if cfg!(windows) { "C:\\foo" } else { "/foo" };

    case(
      &[("justfile", "import 'foo'"), ("foo/bar.just", "")],
      &[Diagnostic {
        display: "invalid import path".into(),
        id: "invalid-import-path".into(),
        ..Diagnostic::error(
          format!("Import path does not exist: `{path}`"),
          lsp::Range::at(0, 7, 0, 12),
        )
      }],
    );
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
