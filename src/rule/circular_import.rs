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

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn import_circular() {
    #[track_caller]
    fn case(justfile: &str, range: lsp::Range) {
      let path = if cfg!(windows) {
        "C:\\justfile"
      } else {
        "/justfile"
      };

      assert_eq!(
        Analyzer {
          config: None,
          view: TestModule::new(&[("justfile", justfile)]).view(),
        }
        .analyze()
        .iter()
        .map(|diagnostic| (diagnostic.message.as_str(), diagnostic.range))
        .collect::<Vec<_>>(),
        [(
          format!("Import `{path}` in `{path}` is circular").as_str(),
          range,
        )],
      );
    }

    case("import 'justfile'", lsp::Range::at(0, 7, 0, 17));
    case("mod foo 'justfile'", lsp::Range::at(0, 8, 0, 18));
  }
}
