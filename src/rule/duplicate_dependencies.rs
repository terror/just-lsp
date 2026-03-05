use super::*;

#[derive(Hash, Eq, PartialEq)]
struct DependencyKey {
  arguments: Vec<String>,
  name: String,
}

define_rule! {
  /// Warns when a recipe lists the same dependency with identical arguments
  /// more than once since `just` only runs it once.
  DuplicateDependenciesRule {
    id: "duplicate-dependencies",
    message: "duplicate dependencies",
    run(context) {
      let mut diagnostics = Vec::new();

      for recipe in context.recipes() {
        let mut seen: HashSet<DependencyKey> = HashSet::new();

        for dependency in &recipe.dependencies {
          let key = DependencyKey {
            name: dependency.name.clone(),
            arguments: dependency
              .arguments
              .iter()
              .map(|argument| argument.value.clone())
              .collect(),
          };

          if !seen.insert(key) {
            let message = if dependency.arguments.is_empty() {
              format!(
                "Recipe `{}` lists dependency `{}` more than once; just only runs it once, so it's redundant",
                recipe.name.value,
                dependency.name
              )
            } else {
              format!(
                "Recipe `{}` lists dependency `{}` with the same arguments more than once; just only runs it once, so it's redundant",
                recipe.name.value,
                dependency.name
              )
            };

            diagnostics.push(Diagnostic::warning(message, dependency.range));
          }
        }
      }

      diagnostics
    }
  }
}
