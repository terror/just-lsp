use super::*;

define_rule! {
  CircularImportRule {
    id: "circular-import",
    title: "circular import",
    severity: Error,
  }
}

impl CircularImportRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
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

        Some(Diagnostic::new(
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn import_circular() {
    #[track_caller]
    fn case(justfile: &str, range: lsp::Range) {
      let path = if cfg!(windows) {
        "C:\\justfile"
      } else {
        "/justfile"
      };

      AnalyzerTest::new(justfile)
        .error(&format!("Import `{path}` in `{path}` is circular"), range)
        .run();
    }

    case("import 'justfile'", lsp::Range::at(0, 7, 0, 17));
    case("mod foo 'justfile'", lsp::Range::at(0, 8, 0, 18));
  }
}
