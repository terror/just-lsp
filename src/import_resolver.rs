use super::*;

use std::path::{Path, PathBuf};

pub(crate) struct ImportedDocument {
  pub(crate) document: Document,
  pub(crate) uri: lsp::Url,
}

pub(crate) struct ImportResolver {
  imported_documents: Vec<ImportedDocument>,
}

impl ImportResolver {
  fn document_dir(document: &Document) -> Option<PathBuf> {
    Self::uri_to_path(&document.uri)
      .ok()
      .and_then(|p| p.parent().map(Path::to_path_buf))
  }

  pub(crate) fn find_recipe(&self, name: &str) -> Option<(lsp::Url, Recipe)> {
    self.imported_documents.iter().find_map(|imported| {
      imported
        .document
        .find_recipe(name)
        .map(|r| (imported.uri.clone(), r))
    })
  }

  pub(crate) fn find_variable(
    &self,
    name: &str,
  ) -> Option<(lsp::Url, Variable)> {
    self.imported_documents.iter().find_map(|imported| {
      imported
        .document
        .find_variable(name)
        .map(|v| (imported.uri.clone(), v))
    })
  }

  pub(crate) fn imported_documents(&self) -> &[ImportedDocument] {
    &self.imported_documents
  }

  pub(crate) fn imported_recipe_names(&self) -> HashSet<String> {
    self
      .imported_documents
      .iter()
      .flat_map(|imported| {
        imported
          .document
          .recipes()
          .into_iter()
          .map(|r| r.name.value)
      })
      .collect()
  }

  pub(crate) fn imported_variable_names(&self) -> HashSet<String> {
    self
      .imported_documents
      .iter()
      .flat_map(|imported| {
        imported
          .document
          .variables()
          .into_iter()
          .map(|v| v.name.value)
      })
      .collect()
  }

  pub(crate) fn new(document: &Document) -> Self {
    let mut visited = HashSet::new();
    let mut imported_documents = Vec::new();

    let base_path = Self::document_dir(document);

    if let Some(base) = &base_path {
      if let Ok(canonical) = Self::uri_to_path(&document.uri)
        .and_then(|p| fs::canonicalize(p).map_err(|e| anyhow!(e)))
      {
        visited.insert(canonical);
      }

      Self::resolve_imports(
        document,
        base,
        &mut visited,
        &mut imported_documents,
      );
    }

    Self { imported_documents }
  }

  fn resolve_imports(
    document: &Document,
    base_dir: &Path,
    visited: &mut HashSet<PathBuf>,
    imported_documents: &mut Vec<ImportedDocument>,
  ) {
    for import in document.imports() {
      let path = base_dir.join(&import.path);

      let Ok(canonical) = fs::canonicalize(&path) else {
        if !import.optional {
          log::warn!("import not found: {}", path.display());
        }
        continue;
      };

      if !visited.insert(canonical.clone()) {
        continue;
      }

      let Ok(content) = fs::read_to_string(&canonical) else {
        continue;
      };

      let Ok(uri) = lsp::Url::from_file_path(&canonical) else {
        continue;
      };

      let mut doc = Document {
        content: Rope::from_str(&content),
        tree: None,
        uri: uri.clone(),
        version: 0,
      };

      if doc.parse().is_err() {
        continue;
      }

      let import_dir = canonical
        .parent()
        .map_or_else(|| base_dir.to_path_buf(), Path::to_path_buf);

      Self::resolve_imports(&doc, &import_dir, visited, imported_documents);

      imported_documents.push(ImportedDocument { document: doc, uri });
    }
  }

  fn uri_to_path(uri: &lsp::Url) -> Result<PathBuf> {
    uri
      .to_file_path()
      .map_err(|()| anyhow!("failed to convert URI to path"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn resolve_basic_import() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("bar.just"), "bar := 'baz'\n").unwrap();

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str("import 'bar.just'\n"),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let resolver = ImportResolver::new(&document);

    assert!(resolver.imported_variable_names().contains("bar"));
  }

  #[test]
  fn resolve_circular_imports() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("a.just"), "import 'b.just'\na := 'foo'\n")
      .unwrap();
    fs::write(dir.path().join("b.just"), "import 'a.just'\nb := 'bar'\n")
      .unwrap();

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str("import 'a.just'\n"),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let resolver = ImportResolver::new(&document);
    let vars = resolver.imported_variable_names();

    assert!(vars.contains("a"));
    assert!(vars.contains("b"));
  }

  #[test]
  fn resolve_imported_recipes() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("bar.just"), "bar:\n  echo bar\n").unwrap();

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str("import 'bar.just'\n"),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let resolver = ImportResolver::new(&document);

    assert!(resolver.imported_recipe_names().contains("bar"));
  }

  #[test]
  fn resolve_nested_imports() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("a.just"), "import 'b.just'\na := 'foo'\n")
      .unwrap();
    fs::write(dir.path().join("b.just"), "b := 'bar'\n").unwrap();

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str("import 'a.just'\n"),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let resolver = ImportResolver::new(&document);
    let vars = resolver.imported_variable_names();

    assert!(vars.contains("a"));
    assert!(vars.contains("b"));
  }

  #[test]
  fn resolve_optional_missing_import() {
    let dir = tempdir().unwrap();

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str("import? 'missing.just'\n"),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let resolver = ImportResolver::new(&document);

    assert!(resolver.imported_variable_names().is_empty());
  }

  #[test]
  fn find_imported_recipe() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("bar.just"), "bar:\n  echo bar\n").unwrap();

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str("import 'bar.just'\n"),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let resolver = ImportResolver::new(&document);

    let (uri, recipe) = resolver.find_recipe("bar").unwrap();
    assert_eq!(recipe.name.value, "bar");
    assert!(uri.path().ends_with("bar.just"));
  }

  #[test]
  fn find_imported_variable() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("bar.just"), "bar := 'baz'\n").unwrap();

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str("import 'bar.just'\n"),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let resolver = ImportResolver::new(&document);

    let (uri, variable) = resolver.find_variable("bar").unwrap();
    assert_eq!(variable.name.value, "bar");
    assert!(uri.path().ends_with("bar.just"));
  }
}
