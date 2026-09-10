use super::*;

enum Item<'a> {
  Alias(&'a Located<Alias>),
  Recipe(&'a Located<Recipe>),
}

impl Item<'_> {
  fn conflict_message(&self, name: &str) -> String {
    match self {
      Item::Alias(_) => format!("Alias `{name}` is redefined as a recipe"),
      Item::Recipe(_) => format!("Recipe `{name}` is redefined as an alias"),
    }
  }

  fn is_same_kind(&self, other: &Self) -> bool {
    matches!(
      (self, other),
      (Item::Alias(_), Item::Alias(_)) | (Item::Recipe(_), Item::Recipe(_))
    )
  }

  fn name(&self) -> &str {
    match self {
      Item::Alias(alias) => &alias.name.value,
      Item::Recipe(recipe) => &recipe.name.value,
    }
  }

  fn range(&self) -> lsp::Range {
    match self {
      Item::Alias(alias) => alias.name.range,
      Item::Recipe(recipe) => recipe.name.range,
    }
  }

  fn uri(&self) -> &lsp::Url {
    match self {
      Item::Alias(alias) => &alias.uri,
      Item::Recipe(recipe) => &recipe.uri,
    }
  }
}

define_rule! {
  /// Reports aliases and recipes that share the same name, since they shadow
  /// each other at runtime.
  AliasRecipeConflictRule {
    id: "alias-recipe-conflict",
    message: "name conflict",
    run(context) {
      let (aliases, recipes) = (context.aliases(), context.recipes());

      if aliases.is_empty() || recipes.is_empty() {
        return Vec::new();
      }

      let items = aliases
        .iter()
        .map(Item::Alias)
        .chain(recipes.iter().map(Item::Recipe))
        .collect::<Vec<_>>();

      let mut imported = HashMap::<&str, Vec<&Item>>::new();

      for item in &items {
        if item.uri() != &context.document().uri {
          imported.entry(item.name()).or_default().push(item);
        }
      }

      let mut local = items
        .iter()
        .filter(|item| item.uri() == &context.document().uri)
        .collect::<Vec<_>>();

      local.sort_by_key(|item| item.range().start);

      local
        .into_iter()
        .fold(
          (HashMap::<&str, &Item>::new(), Vec::new()),
          |(mut seen, mut diagnostics), item| {
            let name = item.name();

            let first = imported
              .get(name)
              .and_then(|items| {
                items.iter().copied().find(|previous| !previous.is_same_kind(item))
              })
              .or_else(|| {
                seen.get(name).copied().filter(|first| !first.is_same_kind(item))
              });

            if let Some(first) = first {
              diagnostics.push(Diagnostic::error(first.conflict_message(name), item.range()));
            }

            seen.entry(name).or_insert(item);

            (seen, diagnostics)
          },
        )
        .1
    }
  }
}
