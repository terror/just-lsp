use super::*;

pub struct RecipeDependencyCycleRule;

impl Rule for RecipeDependencyCycleRule {
  fn id(&self) -> &'static str {
    "recipe-dependency-cycles"
  }

  fn display_name(&self) -> &'static str {
    "Recipe Dependency Cycles"
  }

  fn run(&self, ctx: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let mut dependency_graph = HashMap::new();

    for recipe in ctx.recipes() {
      dependency_graph.insert(
        recipe.name.clone(),
        recipe
          .dependencies
          .iter()
          .map(|dep| dep.name.clone())
          .collect::<Vec<_>>(),
      );
    }

    for recipe in ctx.recipes() {
      let mut visited = HashSet::new();
      let mut path = Vec::new();

      self.detect_cycle(
        &recipe.name,
        &dependency_graph,
        &mut visited,
        &mut path,
        &mut diagnostics,
        ctx,
      );
    }

    diagnostics
  }
}

impl RecipeDependencyCycleRule {
  fn detect_cycle(
    &self,
    recipe_name: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    path: &mut Vec<String>,
    diagnostics: &mut Vec<lsp::Diagnostic>,
    ctx: &RuleContext<'_>,
  ) {
    if visited.contains(recipe_name) {
      return;
    }

    if path.iter().any(|r| r == recipe_name) {
      let cycle_start_idx = path.iter().position(|r| r == recipe_name).unwrap();

      let mut cycle = path[cycle_start_idx..].to_vec();
      cycle.push(recipe_name.to_string());

      if let Some(first) = path.first() {
        if let Some(recipe) = ctx.recipe(first) {
          let message = if cycle.len() == 2 && cycle[0] == cycle[1] {
            format!("Recipe `{}` depends on itself", cycle[0])
          } else if cycle[0] == recipe_name {
            format!(
              "Recipe `{}` has circular dependency `{}`",
              recipe_name,
              cycle.join(" -> ")
            )
          } else {
            path.push(recipe_name.to_string());
            return;
          };

          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: recipe.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message,
            ..Default::default()
          }));
        }
      }

      return;
    }

    if !graph.contains_key(recipe_name) {
      return;
    }

    path.push(recipe_name.to_string());

    if let Some(dependencies) = graph.get(recipe_name) {
      for dependency in dependencies {
        self.detect_cycle(dependency, graph, visited, path, diagnostics, ctx);
      }
    }

    visited.insert(recipe_name.to_string());

    path.pop();
  }
}
