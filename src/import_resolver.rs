use super::*;

use std::path::{Path, PathBuf};

pub(crate) struct ImportedDocument {
  pub(crate) document: Document,
  pub(crate) uri: lsp::Url,
}

pub(crate) struct ImportResolver {
  imported_documents: Vec<ImportedDocument>,
  imported_modules: HashMap<Vec<String>, ImportedDocument>,
}

impl ImportResolver {
  fn document_dir(document: &Document) -> Option<PathBuf> {
    Self::uri_to_path(&document.uri)
      .ok()
      .and_then(|p| p.parent().map(Path::to_path_buf))
  }

  pub(crate) fn find_recipe(&self, name: &str) -> Option<(lsp::Url, Recipe)> {
    let mut split_name: Vec<_> = name.split("::").map(String::from).collect();
    let recipe_name = split_name.pop()?;

    if split_name.is_empty() {
      self.imported_documents.iter().find_map(|imported| {
        imported
          .document
          .find_recipe(&recipe_name)
          .map(|r| (imported.uri.clone(), r))
      })
    } else {
      let imported = self.imported_modules.get(&split_name)?;
      imported
        .document
        .find_recipe(&recipe_name)
        .map(|r| (imported.uri.clone(), r))
    }
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

  pub(crate) fn imported_modules(&self) -> &HashMap<Vec<String>, ImportedDocument> {
    &self.imported_modules
  }

  pub(crate) fn imported_recipe_names(&self) -> HashSet<String> {
    let documents = self.imported_documents.iter().flat_map(|imported| {
      imported
        .document
        .recipes()
        .into_iter()
        .map(|r| r.name.value)
    });
    let modules = self.imported_modules.iter().flat_map(|imported| {
      let path = imported.0.join("::");
      imported
        .1
        .document
        .recipes()
        .into_iter()
        .map(move |r| format!("{}::{}", path, r.name.value))
    });
    documents.chain(modules).collect()
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
    let mut imported_modules = HashMap::new();

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
        &Vec::new(),
        &mut visited,
        &mut imported_documents,
        &mut imported_modules,
      );
    }

    Self {
      imported_documents,
      imported_modules,
    }
  }

  fn resolve_imports(
    document: &Document,
    base_dir: &Path,
    module_path: &Vec<String>,
    visited: &mut HashSet<PathBuf>,
    imported_documents: &mut Vec<ImportedDocument>,
    imported_modules: &mut HashMap<Vec<String>, ImportedDocument>,
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

      Self::resolve_imports(
        &doc,
        &import_dir,
        module_path,
        visited,
        imported_documents,
        imported_modules,
      );

      imported_documents.push(ImportedDocument { document: doc, uri });
    }

    for module in document.modules() {
      let mut inner_module_path = module_path.clone();
      inner_module_path.push(module.name.clone());

      if imported_modules.contains_key(&inner_module_path) {
        continue;
      }

      let path = module
        .path
        .clone()
        .unwrap_or_else(|| format!("{}.just", module.name));
      let path = base_dir.join(&path);

      let Ok(canonical) = fs::canonicalize(&path) else {
        if !module.optional {
          log::warn!("module not found: {}", path.display());
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

      Self::resolve_imports(
        &doc,
        &import_dir,
        &inner_module_path,
        visited,
        imported_documents,
        imported_modules,
      );

      imported_modules
        .insert(inner_module_path, ImportedDocument { document: doc, uri });
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
    fs::write(dir.path().join("baz.just"), "baz:\n  echo baz\n").unwrap();

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str("import \'bar.just\'\nmod baz"),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let resolver = ImportResolver::new(&document);

    let resolved_recipes = resolver.imported_recipe_names();
    assert_eq!(resolved_recipes.len(), 2);
    assert!(resolved_recipes.contains("bar"));
    assert!(resolved_recipes.contains("baz::baz"));
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

  #[test]
  fn find_module_recipe() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("bar.just"), "baz:\n  echo baz\n").unwrap();

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str("mod bar\n"),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let resolver = ImportResolver::new(&document);

    let (uri, recipe) = resolver.find_recipe("bar::baz").unwrap();
    assert_eq!(recipe.name.value, "baz");
    assert!(uri.path().ends_with("bar.just"));
  }

  #[test]
  fn find_imported_module_recipe() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("bar.just"), "bar:\n  echo baz\n").unwrap();
    fs::write(dir.path().join("foo.just"), "mod bar").unwrap();

    let uri = lsp::Url::from_file_path(dir.path().join("justfile")).unwrap();

    let mut document = Document {
      content: Rope::from_str("import \"foo.just\""),
      tree: None,
      uri,
      version: 1,
    };

    document.parse().unwrap();

    let resolver = ImportResolver::new(&document);

    let (uri, recipe) = resolver.find_recipe("bar::bar").unwrap();
    assert_eq!(recipe.name.value, "bar");
    assert!(uri.path().ends_with("bar.just"));
  }
}
