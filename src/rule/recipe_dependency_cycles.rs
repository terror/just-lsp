use super::*;

/// Detects circular dependency chains between recipes to prevent infinite
/// execution loops.
pub(crate) struct RecipeDependencyCycleRule;

impl Rule for RecipeDependencyCycleRule {
  fn id(&self) -> &'static str {
    "recipe-dependency-cycles"
  }

  fn message(&self) -> &'static str {
    "circular dependency"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut dependency_graph = HashMap::new();
    let mut diagnostics = Vec::new();

    for recipe in context.recipes() {
      dependency_graph.insert(
        recipe.name.clone(),
        recipe
          .dependencies
          .iter()
          .map(|dep| dep.name.clone())
          .collect::<Vec<_>>(),
      );
    }

    let mut reported_recipes = HashSet::new();

    for recipe in context.recipes() {
      let mut path = Vec::new();
      let mut visited = HashSet::new();

      let mut traversal_state = TraversalState {
        visited: &mut visited,
        path: &mut path,
        reported_recipes: &mut reported_recipes,
      };

      self.detect_cycle(
        &recipe.name,
        &dependency_graph,
        &mut diagnostics,
        context,
        &mut traversal_state,
      );
    }

    diagnostics
  }
}

struct TraversalState<'a> {
  path: &'a mut Vec<String>,
  reported_recipes: &'a mut HashSet<String>,
  visited: &'a mut HashSet<String>,
}

impl RecipeDependencyCycleRule {
  fn detect_cycle(
    &self,
    recipe_name: &str,
    graph: &HashMap<String, Vec<String>>,
    diagnostics: &mut Vec<Diagnostic>,
    context: &RuleContext<'_>,
    traversal: &mut TraversalState<'_>,
  ) {
    if traversal.visited.contains(recipe_name) {
      return;
    }

    if traversal.path.iter().any(|r| r == recipe_name) {
      let cycle_start_idx = traversal
        .path
        .iter()
        .position(|r| r == recipe_name)
        .unwrap();

      let mut cycle = traversal.path[cycle_start_idx..].to_vec();
      cycle.push(recipe_name.to_string());

      if let Some(recipe) = context.recipe(recipe_name) {
        let message = if cycle.len() == 2 && cycle[0] == cycle[1] {
          format!("Recipe `{}` depends on itself", cycle[0])
        } else if cycle[0] == recipe_name {
          format!(
            "Recipe `{}` has circular dependency `{}`",
            recipe_name,
            cycle.join(" -> ")
          )
        } else {
          traversal.path.push(recipe_name.to_string());
          return;
        };

        if !traversal.reported_recipes.insert(recipe_name.to_string()) {
          return;
        }

        diagnostics.push(Diagnostic::error(message, recipe.range));
      }

      return;
    }

    if !graph.contains_key(recipe_name) {
      return;
    }

    traversal.path.push(recipe_name.to_string());

    if let Some(dependencies) = graph.get(recipe_name) {
      for dependency in dependencies {
        self.detect_cycle(dependency, graph, diagnostics, context, traversal);
      }
    }

    traversal.visited.insert(recipe_name.to_string());

    traversal.path.pop();
  }
}
