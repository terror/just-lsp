use super::*;

enum Item<'a> {
  Alias(&'a Alias, &'a GroupSet),
  Recipe(&'a Recipe, &'a GroupSet),
}

impl Item<'_> {
  fn conflict_message(&self, name: &str) -> String {
    match self {
      Item::Alias(..) => format!("Recipe `{name}` is redefined as an alias"),
      Item::Recipe(..) => format!("Alias `{name}` is redefined as a recipe"),
    }
  }

  fn groups(&self) -> &GroupSet {
    match self {
      Item::Alias(_, groups) | Item::Recipe(_, groups) => groups,
    }
  }

  fn name(&self) -> &str {
    match self {
      Item::Alias(alias, _) => &alias.name.value,
      Item::Recipe(recipe, _) => &recipe.name.value,
    }
  }

  fn range(&self) -> lsp::Range {
    match self {
      Item::Alias(alias, _) => alias.name.range,
      Item::Recipe(recipe, _) => recipe.name.range,
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
      let (aliases, recipes) = (
        context.aliases_with_groups(),
        context.recipes_with_groups(),
      );

      let mut items = aliases
        .map(|(alias, groups)| Item::Alias(alias, groups))
        .chain(recipes.map(|(recipe, groups)| Item::Recipe(recipe, groups)))
        .collect::<Vec<_>>();

      items.sort_by_key(|item| {
        let range = item.range();
        (range.start.line, range.start.character)
      });

      let mut aliases = HashMap::<&str, GroupSet>::new();
      let mut recipes = HashMap::<&str, GroupSet>::new();
      let mut diagnostics = Vec::new();

      for item in &items {
        let name = item.name();

        let opposite = match item {
          Item::Alias(..) => &recipes,
          Item::Recipe(..) => &aliases,
        };

        if opposite
          .get(name)
          .is_some_and(|groups| groups.conflicts_with(item.groups()))
        {
          diagnostics.push(Diagnostic::error(
            item.conflict_message(name),
            item.range(),
          ));
        }

        match item {
          Item::Alias(..) => aliases
            .entry(name)
            .or_default()
            .union_with(item.groups().clone()),
          Item::Recipe(..) => recipes
            .entry(name)
            .or_default()
            .union_with(item.groups().clone()),
        }
      }

      diagnostics
    }
  }
}
