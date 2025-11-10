use super::*;

/// Detects recipes that have the same name and overlapping OS constraints, which
/// would shadow each other at runtime unless overrides are enabled.
pub struct DuplicateRecipeRule;

impl Rule for DuplicateRecipeRule {
  fn display_name(&self) -> &'static str {
    "Duplicate Recipes"
  }

  fn id(&self) -> &'static str {
    "duplicate-recipes"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let allow_duplicates = context.settings().iter().any(|setting| {
      setting.name == "allow-duplicate-recipes"
        && matches!(setting.kind, SettingKind::Boolean(true))
    });

    if allow_duplicates {
      return Vec::new();
    }

    let mut diagnostics = Vec::new();

    let mut recipe_groups: HashMap<String, Vec<(lsp::Range, HashSet<Group>)>> =
      HashMap::new();

    for recipe in context.recipes() {
      recipe_groups
        .entry(recipe.name.clone())
        .or_default()
        .push((recipe.range, recipe.groups()));
    }

    for (recipe_name, group) in &recipe_groups {
      if group.len() <= 1 {
        continue;
      }

      for (i, (range, a)) in group.iter().enumerate() {
        for (_, (_, b)) in group.iter().enumerate().take(i) {
          let has_conflict =
            a.iter().any(|a| b.iter().any(|b| a.conflicts_with(b)));

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
