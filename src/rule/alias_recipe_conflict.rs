use super::*;

enum Item<'a> {
  Alias(&'a Located<Alias>),
  Recipe(&'a Located<Recipe>),
}

impl<'a> Item<'a> {
  fn conflict_message(&self, name: &str) -> String {
    match self {
      Item::Alias(_) => format!("Alias `{name}` is redefined as a recipe"),
      Item::Recipe(_) => format!("Recipe `{name}` is redefined as an alias"),
    }
  }

  fn conflicts_with(&self, other: &Self) -> bool {
    !self.is_same_kind(other) && self.groups().conflicts_with(&other.groups())
  }

  fn groups(&self) -> GroupSet {
    match self {
      Item::Alias(alias) => GroupSet::from_attributes(&alias.attributes),
      Item::Recipe(recipe) => GroupSet::from_attributes(&recipe.attributes),
    }
  }

  fn is_same_kind(&self, other: &Self) -> bool {
    matches!(
      (self, other),
      (Item::Alias(_), Item::Alias(_)) | (Item::Recipe(_), Item::Recipe(_))
    )
  }

  fn name(&self) -> &'a str {
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
    title: "name conflict",
    severity: Error,
  }
}

impl AliasRecipeConflictRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let module = context.module();

    if module.aliases().is_empty() || module.recipes().is_empty() {
      return Vec::new();
    }

    let (imported, local) = (context.imported(), context.local());

    let mut imported_items = HashMap::<&str, Vec<Item>>::new();

    for item in imported
      .aliases()
      .map(Item::Alias)
      .chain(imported.recipes().map(Item::Recipe))
    {
      imported_items.entry(item.name()).or_default().push(item);
    }

    let mut local = local
      .aliases()
      .map(Item::Alias)
      .chain(local.recipes().map(Item::Recipe))
      .collect::<Vec<_>>();

    local.sort_by_key(|item| item.range().start);

    local
      .iter()
      .fold(
        (HashMap::<&str, Vec<&Item>>::new(), Vec::new()),
        |(mut seen, mut diagnostics), item| {
          let name = item.name();

          let first = imported_items
            .get(name)
            .and_then(|items| {
              items.iter().find(|previous| previous.conflicts_with(item))
            })
            .or_else(|| {
              seen.get(name).and_then(|items| {
                items
                  .iter()
                  .copied()
                  .find(|previous| previous.conflicts_with(item))
              })
            });

          if let Some(first) = first {
            diagnostics.push(Diagnostic::new(
              first.conflict_message(name),
              item.range(),
            ));
          }

          seen.entry(name).or_default().push(item);

          (seen, diagnostics)
        },
      )
      .1
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn alias_recipe_conflict_alias_then_recipe() {
    AnalyzerTest::new(indoc! {
      "
      alias t := other

      other:
        echo \"other\"

      t:
        echo \"recipe\"
      "
    })
    .error(
      "Alias `t` is redefined as a recipe",
      lsp::Range::at(5, 0, 5, 1),
    )
    .run();
  }

  #[test]
  fn alias_recipe_conflict_recipe_then_alias() {
    AnalyzerTest::new(indoc! {
      "
      other:
        echo \"other\"

      t:
        echo \"recipe\"

      alias t := other
      "
    })
    .error(
      "Recipe `t` is redefined as an alias",
      lsp::Range::at(6, 6, 6, 7),
    )
    .run();
  }

  #[test]
  fn alias_recipe_conflict_respects_os_attributes() {
    AnalyzerTest::new("[windows]\nalias foo := bar\n[unix]\nfoo:\nbar:\n")
      .run();

    AnalyzerTest::new(
      "[windows]\nalias foo := bar\n[unix]\nalias foo := bar\n[linux]\nfoo:\nbar:\n",
    )
    .error(
      "Alias `foo` is redefined as a recipe",
      lsp::Range::at(5, 0, 5, 3),
    )
    .run();
  }

  #[test]
  fn analyzer_ignores_imported_alias_recipe_conflicts() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "foo:\nalias foo := bar\n")
      .run();
  }

  #[test]
  fn analyzer_imported_alias_conflict_reports_local_recipe() {
    AnalyzerTest::new("foo:\nimport 'foo.just'\n")
      .file("foo.just", "\n\nalias foo := bar\nbar:\n")
      .error(
        "Alias `foo` is redefined as a recipe",
        lsp::Range::at(0, 0, 0, 3),
      )
      .run();
  }

  #[test]
  fn analyzer_imported_recipe_conflict_reports_local_alias() {
    AnalyzerTest::new("alias foo := bar\nimport 'foo.just'\n")
      .file("foo.just", "\n\nfoo:\nbar:\n")
      .error(
        "Recipe `foo` is redefined as an alias",
        lsp::Range::at(0, 6, 0, 9),
      )
      .run();
  }
}
