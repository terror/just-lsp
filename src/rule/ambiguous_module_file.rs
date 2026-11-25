use super::*;

define_rule! {
  AmbiguousModuleFileRule {
    id: "ambiguous-module-file",
    message: "ambiguous module file",
    run(context) {
      let document = context.document();

      let Some(directory) = document
        .uri
        .file_path()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
      else {
        return Vec::new();
      };

      document
        .modules()
        .into_iter()
        .filter(ModuleDeclaration::is_enabled)
        .filter_map(|module| {
          let mut files = module.files(&document.uri, context.view().filesystem());

          if files.len() < 2 {
            return None;
          }

          files.sort();

          let files = files
            .iter()
            .map(|file| file.strip_prefix(&directory).unwrap_or(file).display().to_string())
            .collect::<Vec<_>>();

          Some(Diagnostic::error(
            format!(
              "Found multiple source files for module `{}`: {}",
              module.name.value,
              List::and_ticked(&files),
            ),
            module.location(),
          ))
        })
        .collect()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn module_ambiguous_files() {
    AnalyzerTest::new("mod? foo")
      .file("foo.just", "")
      .file("foo/mod.just", "")
      .error(
        &"Found multiple source files for module `foo`: `foo/mod.just` and `foo.just`"
          .replace('/', std::path::MAIN_SEPARATOR_STR),
        lsp::Range::at(0, 5, 0, 8),
      )
      .run();

    AnalyzerTest::new("mod foo 'bar'")
      .file("bar/mod.just", "")
      .file("bar/justfile", "")
      .file("bar/.justfile", "")
      .error(
        &"Found multiple source files for module `foo`: `bar/.justfile`, `bar/justfile`, and `bar/mod.just`"
          .replace('/', std::path::MAIN_SEPARATOR_STR),
        lsp::Range::at(0, 8, 0, 13),
      )
      .run();
  }
}
