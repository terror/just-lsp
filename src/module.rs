use super::*;

#[derive(Debug)]
pub(crate) struct Module {
  bindings: HashMap<lsp::Url, OnceLock<Arc<Bindings>>>,
  pub(crate) children: BTreeMap<String, lsp::Url>,
  pub(crate) import_scope: ImportScope,
  symbols: OnceLock<Arc<SymbolTable>>,
  usage: OnceLock<Arc<Usage>>,
}

impl Module {
  pub(crate) fn bindings(
    &self,
    view: &ModuleView,
    document: &Document,
  ) -> Option<Arc<Bindings>> {
    Some(
      self
        .bindings
        .get(&document.uri)?
        .get_or_init(|| Arc::new(Bindings::new(view, document)))
        .clone(),
    )
  }

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

        let ProjectDependencyTarget::Path(target) = &dependency.target else {
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
      bindings: import_scope
        .documents()
        .iter()
        .map(|document| (document.document.clone(), OnceLock::new()))
        .collect(),
      children,
      import_scope,
      symbols: OnceLock::new(),
      usage: OnceLock::new(),
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

  pub(crate) fn usage(&self, view: &ModuleView) -> Arc<Usage> {
    self
      .usage
      .get_or_init(|| Arc::new(Usage::new(view)))
      .clone()
  }
}
