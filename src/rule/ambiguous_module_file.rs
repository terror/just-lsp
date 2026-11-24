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
