use super::*;

pub struct DuplicateRecipeRule;

impl Rule for DuplicateRecipeRule {
  fn id(&self) -> &'static str {
    "duplicate-recipes"
  }

  fn display_name(&self) -> &'static str {
    "Duplicate Recipes"
  }

  fn run(&self, ctx: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let allow_duplicates = ctx.settings().iter().any(|setting| {
      setting.name == "allow-duplicate-recipes"
        && matches!(setting.kind, SettingKind::Boolean(true))
    });

    if allow_duplicates {
      return Vec::new();
    }

    let mut diagnostics = Vec::new();

    let mut recipe_groups: HashMap<
      String,
      Vec<(lsp::Range, HashSet<OsGroup>)>,
    > = HashMap::new();

    for recipe in ctx.recipes() {
      recipe_groups
        .entry(recipe.name.clone())
        .or_default()
        .push((recipe.range, recipe.os_groups()));
    }

    for (recipe_name, group) in &recipe_groups {
      if group.len() <= 1 {
        continue;
      }

      for (i, (range, os_groups1)) in group.iter().enumerate() {
        for (_, (_, os_groups2)) in group.iter().enumerate().take(i) {
          let has_conflict = os_groups1.iter().any(|group1| {
            os_groups2
              .iter()
              .any(|group2| group1.conflicts_with(group2))
          });

          if has_conflict {
            diagnostics.push(self.diagnostic(lsp::Diagnostic {
              range: *range,
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              message: format!("Duplicate recipe name `{recipe_name}`"),
              ..Default::default()
            }));

            break;
          }
        }
      }
    }

    diagnostics
  }
}
