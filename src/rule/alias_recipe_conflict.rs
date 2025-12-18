use super::*;

enum Item<'a> {
  Alias(&'a Alias),
  Recipe(&'a Recipe),
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

      let mut items = aliases
        .iter()
        .map(Item::Alias)
        .chain(recipes.iter().map(Item::Recipe))
        .collect::<Vec<_>>();

      items.sort_by_key(|item| {
        let range = item.range();
        (range.start.line, range.start.character)
      });

      items
        .iter()
        .fold(
          (HashMap::<&str, &Item>::new(), Vec::new()),
          |(mut seen, mut diagnostics), item| {
            let name = item.name();

            match seen.get(name) {
              Some(first) if !first.is_same_kind(item) => {
                diagnostics.push(Diagnostic::error(first.conflict_message(name), item.range()));
              }
              None => {
                seen.insert(name, item);
              }
              _ => {}
            }
            (seen, diagnostics)
          },
        )
        .1
    }
  }
}
