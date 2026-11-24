use super::*;

#[derive(Debug)]
pub(crate) struct SymbolTable {
  pub(crate) aliases: Vec<Located<Alias>>,
  duplicates: OnceLock<Duplicates>,
  pub(crate) functions: Vec<Located<Function>>,
  pub(crate) modules: Vec<Located<ModuleDeclaration>>,
  pub(crate) recipes: Vec<Located<Recipe>>,
  pub(crate) resolved_functions: HashMap<String, usize>,
  pub(crate) resolved_recipes: HashMap<String, usize>,
  pub(crate) resolved_variables: HashMap<String, usize>,
  pub(crate) settings: Vec<Located<Setting>>,
  pub(crate) unexports: Vec<Located<Unexport>>,
  pub(crate) variables: Vec<Located<Variable>>,
}

impl SymbolTable {
  fn declarations<T>(
    documents: &[ScopeDocument<&Document>],
    declarations: impl Fn(&Document) -> Vec<T>,
  ) -> Vec<Located<T>> {
    documents
      .iter()
      .flat_map(|document| {
        declarations(document.document)
          .into_iter()
          .map(|declaration| {
            Located::new(document.document.uri.clone(), declaration)
          })
      })
      .collect()
  }

  pub(crate) fn duplicates(&self) -> &Duplicates {
    self.duplicates.get_or_init(|| Duplicates::new(self))
  }

  pub(crate) fn new(documents: &[ScopeDocument<&Document>]) -> Self {
    let functions = Self::declarations(documents, Document::functions);
    let recipes = Self::declarations(documents, Document::recipes);
    let variables = Self::declarations(documents, Document::variables);

    Self {
      aliases: Self::declarations(documents, Document::aliases),
      duplicates: OnceLock::new(),
      resolved_functions: Self::resolve(
        documents,
        &functions,
        |function| &function.name.value,
        |function| function.range.start,
      ),
      resolved_recipes: Self::resolve(
        documents,
        &recipes,
        |recipe| &recipe.name.value,
        |recipe| recipe.range.start,
      ),
      resolved_variables: Self::resolve(
        documents,
        &variables,
        |variable| &variable.name.value,
        |variable| variable.range.start,
      ),
      functions,
      modules: Self::declarations(documents, Document::modules),
      recipes,
      settings: Self::declarations(documents, Document::settings),
      unexports: Self::declarations(documents, Document::unexports),
      variables,
    }
  }

  fn resolve<T>(
    documents: &[ScopeDocument<&Document>],
    declarations: &[Located<T>],
    name: impl Fn(&T) -> &str,
    position: impl Fn(&T) -> lsp::Position,
  ) -> HashMap<String, usize> {
    let precedence = documents
      .iter()
      .enumerate()
      .map(|(order, document)| {
        (
          &document.document.uri,
          (Reverse(document.load_depth), order),
        )
      })
      .collect::<HashMap<_, _>>();

    let mut candidates = (0..declarations.len()).collect::<Vec<_>>();

    candidates.sort_by_key(|index| {
      let declaration = &declarations[*index];

      (precedence[&declaration.uri], position(declaration))
    });

    candidates
      .into_iter()
      .map(|index| (name(&declarations[index]).to_owned(), index))
      .collect()
  }
}
