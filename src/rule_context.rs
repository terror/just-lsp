use super::*;

pub struct UnresolvedIdentifier {
  pub name: String,
  pub range: lsp::Range,
}

#[derive(Default)]
pub struct IdentifierAnalysis {
  recipe_identifier_usage: HashMap<String, HashSet<String>>,
  unresolved_identifiers: Vec<UnresolvedIdentifier>,
  variable_usage: HashMap<String, bool>,
}

impl IdentifierAnalysis {
  fn new(context: &RuleContext<'_>) -> Self {
    let mut variable_usage = context
      .variables()
      .iter()
      .map(|variable| (variable.name.value.clone(), false))
      .collect::<HashMap<_, _>>();

    let mut recipe_identifier_usage = context
      .recipes()
      .iter()
      .map(|recipe| (recipe.name.clone(), HashSet::new()))
      .collect::<HashMap<_, _>>();

    let mut unresolved_identifiers = Vec::new();

    if let Some(tree) = context.tree() {
      let root = tree.root_node();

      for identifier in root.find_all("expression > value > identifier") {
        Self::record_identifier(
          context,
          &mut recipe_identifier_usage,
          &mut variable_usage,
          &mut unresolved_identifiers,
          identifier,
        );
      }

      for identifier in root.find_all("parameter > value > identifier") {
        Self::record_identifier(
          context,
          &mut recipe_identifier_usage,
          &mut variable_usage,
          &mut unresolved_identifiers,
          identifier,
        );
      }
    }

    Self {
      recipe_identifier_usage,
      unresolved_identifiers,
      variable_usage,
    }
  }

  fn record_identifier(
    context: &RuleContext<'_>,
    recipe_identifier_usage: &mut HashMap<String, HashSet<String>>,
    variable_usage: &mut HashMap<String, bool>,
    unresolved_identifiers: &mut Vec<UnresolvedIdentifier>,
    identifier: Node<'_>,
  ) {
    let document = context.document();
    let recipe_parameters = context.recipe_parameters();
    let value_names = context.variable_and_builtin_names();

    let recipe_name = identifier
      .get_parent("recipe")
      .as_ref()
      .and_then(|recipe_node| recipe_node.find("recipe_header > identifier"))
      .map_or_else(String::new, |identifier_node| {
        document.get_node_text(&identifier_node)
      });

    let identifier_name = document.get_node_text(&identifier);

    if let Some(recipe) = context.recipe(&recipe_name) {
      recipe_identifier_usage
        .entry(recipe.name.clone())
        .or_default()
        .insert(identifier_name.clone());

      if recipe_parameters.get(&recipe.name).is_some_and(|params| {
        params.iter().any(|param| param.name == identifier_name)
      }) {
        return;
      }
    }

    if let Some(usage) = variable_usage.get_mut(&identifier_name) {
      *usage = true;
    }

    if !value_names.contains(&identifier_name) {
      unresolved_identifiers.push(UnresolvedIdentifier {
        name: identifier_name,
        range: identifier.get_range(),
      });
    }
  }
}

pub struct RuleContext<'a> {
  aliases: OnceCell<Vec<Alias>>,
  attributes: OnceCell<Vec<Attribute>>,
  document: &'a Document,
  document_variable_names: OnceCell<HashSet<String>>,
  function_calls: OnceCell<Vec<FunctionCall>>,
  identifier_analysis: OnceCell<IdentifierAnalysis>,
  recipe_names: OnceCell<HashSet<String>>,
  recipe_parameters: OnceCell<HashMap<String, Vec<Parameter>>>,
  recipes: OnceCell<Vec<Recipe>>,
  settings: OnceCell<Vec<Setting>>,
  variable_and_builtin_names: OnceCell<HashSet<String>>,
  variables: OnceCell<Vec<Variable>>,
}

impl<'a> RuleContext<'a> {
  pub fn aliases(&self) -> &[Alias] {
    self
      .aliases
      .get_or_init(|| self.document.aliases())
      .as_slice()
  }

  pub fn attributes(&self) -> &[Attribute] {
    self
      .attributes
      .get_or_init(|| self.document.attributes())
      .as_slice()
  }

  pub fn document(&self) -> &Document {
    self.document
  }

  pub fn document_variable_names(&self) -> &HashSet<String> {
    self.document_variable_names.get_or_init(|| {
      self
        .variables()
        .iter()
        .map(|variable| variable.name.value.clone())
        .collect()
    })
  }

  pub fn function_calls(&self) -> &[FunctionCall] {
    self
      .function_calls
      .get_or_init(|| self.document.function_calls())
      .as_slice()
  }

  fn identifier_analysis(&self) -> &IdentifierAnalysis {
    self
      .identifier_analysis
      .get_or_init(|| IdentifierAnalysis::new(self))
  }

  pub fn new(document: &'a Document) -> Self {
    Self {
      aliases: OnceCell::new(),
      attributes: OnceCell::new(),
      document,
      document_variable_names: OnceCell::new(),
      function_calls: OnceCell::new(),
      identifier_analysis: OnceCell::new(),
      recipe_names: OnceCell::new(),
      recipe_parameters: OnceCell::new(),
      recipes: OnceCell::new(),
      settings: OnceCell::new(),
      variable_and_builtin_names: OnceCell::new(),
      variables: OnceCell::new(),
    }
  }

  pub fn recipe(&self, name: &str) -> Option<&Recipe> {
    self.recipes().iter().find(|recipe| recipe.name == name)
  }

  pub fn recipe_identifier_usage(&self) -> &HashMap<String, HashSet<String>> {
    &self.identifier_analysis().recipe_identifier_usage
  }

  pub fn recipe_names(&self) -> &HashSet<String> {
    self
      .recipe_names
      .get_or_init(|| self.recipes().iter().map(|r| r.name.clone()).collect())
  }

  pub fn recipe_parameters(&self) -> &HashMap<String, Vec<Parameter>> {
    self.recipe_parameters.get_or_init(|| {
      self
        .recipes()
        .iter()
        .map(|recipe| (recipe.name.clone(), recipe.parameters.clone()))
        .collect()
    })
  }

  pub fn recipes(&self) -> &[Recipe] {
    self
      .recipes
      .get_or_init(|| self.document.recipes())
      .as_slice()
  }

  pub fn settings(&self) -> &[Setting] {
    self
      .settings
      .get_or_init(|| self.document.settings())
      .as_slice()
  }

  pub fn tree(&self) -> Option<&Tree> {
    self.document.tree.as_ref()
  }

  pub fn unresolved_identifiers(&self) -> &[UnresolvedIdentifier] {
    &self.identifier_analysis().unresolved_identifiers
  }

  pub fn variable_and_builtin_names(&self) -> &HashSet<String> {
    self.variable_and_builtin_names.get_or_init(|| {
      let mut names = self.document_variable_names().clone();

      names.extend(BUILTINS.into_iter().filter_map(|builtin| match builtin {
        Builtin::Constant { name, .. } => Some(name.to_owned()),
        _ => None,
      }));

      names
    })
  }

  pub fn variable_usage(&self) -> &HashMap<String, bool> {
    &self.identifier_analysis().variable_usage
  }

  pub fn variables(&self) -> &[Variable] {
    self
      .variables
      .get_or_init(|| self.document.variables())
      .as_slice()
  }
}
