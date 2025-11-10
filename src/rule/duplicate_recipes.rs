use super::*;

/// Detects recipes that have the same name and overlapping OS constraints, which
/// would shadow each other at runtime unless overrides are enabled.
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

    let mut recipe_groups: HashMap<String, Vec<(lsp::Range, HashSet<Group>)>> =
      HashMap::new();

    for recipe in ctx.recipes() {
      recipe_groups
        .entry(recipe.name.clone())
        .or_default()
        .push((recipe.range, recipe.groups()));
    }

    for (recipe_name, group) in &recipe_groups {
      if group.len() <= 1 {
        continue;
      }

      for (i, (range, groups1)) in group.iter().enumerate() {
        for (_, (_, groups2)) in group.iter().enumerate().take(i) {
          let has_conflict = groups1.iter().any(|group1| {
            groups2.iter().any(|group2| group1.conflicts_with(group2))
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
