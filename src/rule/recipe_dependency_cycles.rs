use super::*;

define_rule! {
  /// Detects circular dependency chains between recipes to prevent infinite
  /// execution loops.
  RecipeDependencyCycleRule {
    id: "recipe-dependency-cycles",
    message: "circular dependency",
    run(context) {
      let mut dependency_graph = HashMap::new();

      for recipe in context.recipes() {
        dependency_graph.insert(
          recipe.name.value.clone(),
          recipe
            .dependencies
            .iter()
            .map(|dep| dep.name.value.clone())
            .collect::<Vec<_>>(),
        );
      }

      let mut traversal = TraversalState {
        context,
        diagnostics: Vec::new(),
        graph: &dependency_graph,
        path: Vec::new(),
        reported_recipes: HashSet::new(),
        visited: HashSet::new(),
      };

      for recipe in context.recipes() {
        traversal.visited.clear();
        traversal.detect_cycle(&recipe.name.value);
      }

      traversal.diagnostics
    }
  }
}

struct TraversalState<'a, 'b> {
  context: &'a RuleContext<'b>,
  diagnostics: Vec<Diagnostic>,
  graph: &'a HashMap<String, Vec<String>>,
  path: Vec<String>,
  reported_recipes: HashSet<String>,
  visited: HashSet<String>,
}

impl TraversalState<'_, '_> {
  fn detect_cycle(&mut self, recipe_name: &str) {
    if self.visited.contains(recipe_name) {
      return;
    }

    if let Some(cycle_start_idx) =
      self.path.iter().position(|r| r == recipe_name)
    {
      let mut cycle = self.path[cycle_start_idx..].to_vec();
      cycle.push(recipe_name.to_string());

      if let Some(recipe) = self.context.recipe(recipe_name) {
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

    let Some(dependencies) = self.graph.get(recipe_name) else {
      return;
    };

    self.path.push(recipe_name.to_string());

    for dependency in dependencies {
      self.detect_cycle(dependency);
    }

    self.visited.insert(recipe_name.to_string());

    self.path.pop();
  }
}
