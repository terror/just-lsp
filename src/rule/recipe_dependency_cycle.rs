use super::*;

define_rule! {
  /// Detects circular dependency chains between recipes to prevent infinite
  /// execution loops.
  RecipeDependencyCycleRule {
    id: "recipe-dependency-cycle",
    message: "circular dependency",
    run(context) {
      let recipes = context.view().resolved_recipes();

      let mut traversal = TraversalState {
        diagnostics: Vec::new(),
        document: context.document(),
        path: Vec::new(),
        recipes: &recipes,
        reported_recipes: HashSet::new(),
        visited: HashSet::new(),
      };

      for recipe in context.document().recipes() {
        traversal.visited.clear();
        traversal.detect_cycle(&recipe.name.value);
      }

      traversal.diagnostics
    }
  }
}

struct TraversalState<'a> {
  diagnostics: Vec<Diagnostic>,
  document: &'a Document,
  path: Vec<String>,
  recipes: &'a HashMap<String, Located<Recipe>>,
  reported_recipes: HashSet<String>,
  visited: HashSet<String>,
}

impl TraversalState<'_> {
  fn detect_cycle(&mut self, recipe_name: &str) {
    if self.visited.contains(recipe_name) {
      return;
    }

    let Some(recipe) = self.recipes.get(recipe_name) else {
      return;
    };

    if let Some(cycle_start_idx) =
      self.path.iter().position(|r| r == recipe_name)
    {
      let mut cycle = self.path[cycle_start_idx..].to_vec();
      cycle.push(recipe_name.to_string());

      if recipe.uri == self.document.uri {
        let message = if cycle.len() == 2 {
          format!("Recipe `{recipe_name}` depends on itself")
        } else {
          format!(
            "Recipe `{recipe_name}` has circular dependency `{}`",
            cycle.join(" -> ")
          )
        };

        if !self.reported_recipes.insert(recipe_name.to_string()) {
          return;
        }

        self
          .diagnostics
          .push(Diagnostic::error(message, recipe.range));
      }

      return;
    }

    self.path.push(recipe_name.to_string());

    for dependency in &recipe.dependencies {
      self.detect_cycle(&dependency.name.value);
    }

    self.visited.insert(recipe_name.to_string());

    self.path.pop();
  }
}
