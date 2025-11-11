use super::*;

/// Reports aliases and recipes that share the same name, since they shadow
/// each other at runtime.
pub(crate) struct AliasRecipeConflictRule;

impl Rule for AliasRecipeConflictRule {
  fn display_name(&self) -> &'static str {
    "Alias/Recipe Conflicts"
  }

  fn id(&self) -> &'static str {
    "alias-recipe-conflict"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let (aliases, recipes) = (context.aliases(), context.recipes());

    if aliases.is_empty() || recipes.is_empty() {
      return Vec::new();
    }

    let recipe_name_lookup = Self::recipe_name_ranges(context);

    let recipe_name_ranges = recipes
      .iter()
      .map(|recipe| {
        recipe_name_lookup
          .get(&RangeKey::from(recipe.range))
          .copied()
          .unwrap_or(recipe.range)
      })
      .collect::<Vec<_>>();

    let mut first_alias_index: HashMap<String, usize> = HashMap::new();

    for (index, alias) in aliases.iter().enumerate() {
      first_alias_index
        .entry(alias.name.value.clone())
        .or_insert(index);
    }

    let mut first_recipe_index: HashMap<String, usize> = HashMap::new();

    for (index, recipe) in recipes.iter().enumerate() {
      first_recipe_index
        .entry(recipe.name.clone())
        .or_insert(index);
    }

    let mut diagnostics = Vec::new();

    for alias in aliases {
      if let Some(&recipe_index) = first_recipe_index.get(&alias.name.value) {
        let recipe_range = recipe_name_ranges[recipe_index];

        if Self::is_after(&alias.name.range, &recipe_range) {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: alias.name.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!(
              "Recipe `{}` is redefined as an alias",
              recipes[recipe_index].name
            ),
            ..Default::default()
          }));
        }
      }
    }

    for (index, recipe) in recipes.iter().enumerate() {
      if let Some(&alias_index) = first_alias_index.get(&recipe.name) {
        let (recipe_range, alias_range) =
          (recipe_name_ranges[index], aliases[alias_index].name.range);

        if Self::is_after(&recipe_range, &alias_range) {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: recipe_range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!(
              "Alias `{}` is redefined as a recipe",
              recipe.name
            ),
            ..Default::default()
          }));
        }
      }
    }

    diagnostics
  }
}

impl AliasRecipeConflictRule {
  fn is_after(a: &lsp::Range, b: &lsp::Range) -> bool {
    (a.start.line, a.start.character) > (b.start.line, b.start.character)
  }

  fn recipe_name_ranges(
    context: &RuleContext<'_>,
  ) -> HashMap<RangeKey, lsp::Range> {
    let mut lookup = HashMap::new();

    if let Some(tree) = context.tree() {
      for recipe_node in tree.root_node().find_all("recipe") {
        if let Some(name_node) = recipe_node.find("recipe_header > identifier")
        {
          lookup.insert(
            RangeKey::from(recipe_node.get_range()),
            name_node.get_range(),
          );
        }
      }
    }

    lookup
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct RangeKey {
  end_character: u32,
  end_line: u32,
  start_character: u32,
  start_line: u32,
}

impl From<lsp::Range> for RangeKey {
  fn from(range: lsp::Range) -> Self {
    Self {
      start_line: range.start.line,
      start_character: range.start.character,
      end_line: range.end.line,
      end_character: range.end.character,
    }
  }
}
