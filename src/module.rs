use super::*;

#[derive(Debug)]
pub(crate) struct Module {
  pub(crate) children: BTreeMap<String, lsp::Url>,
  pub(crate) import_scope: ImportScope,
  symbols: OnceLock<Arc<SymbolTable>>,
}

impl Module {
  pub(crate) fn new(
    root: &lsp::Url,
    dependencies: &HashMap<lsp::Url, Vec<ProjectDependency>>,
  ) -> Self {
    let import_scope = ImportScope::new(root, dependencies);

    let mut children = BTreeMap::new();

    for document in import_scope.documents() {
      for dependency in
        dependencies.get(&document.document).into_iter().flatten()
      {
        let ProjectDependencyKind::Module { name } = &dependency.kind else {
          continue;
        };

        let ProjectDependencyTarget::Resolved(target) = &dependency.target
        else {
          continue;
        };

        if dependencies.contains_key(target) {
          children
            .entry(name.clone())
            .or_insert_with(|| target.clone());
        }
      }
    }

    Self {
      children,
      import_scope,
      symbols: OnceLock::new(),
    }
  }

  pub(crate) fn symbols(
    &self,
    documents: &[ScopeDocument<&Document>],
  ) -> Arc<SymbolTable> {
    self
      .symbols
      .get_or_init(|| Arc::new(SymbolTable::new(documents)))
      .clone()
  }
}
