use super::*;

define_rule! {
  /// Reports recipe dependencies that reference recipes which don't exist in
  /// the current document.
  UnresolvedDependencyRule {
    id: "unresolved-dependency",
    message: "missing dependency",
    run(context) {
      context
        .local_declarations(context.recipes())
        .flat_map(|recipe| &recipe.dependencies)
        .filter_map(|dependency| {
          context.unresolved_recipe(&dependency.name, dependency.range)
        })
        .collect()
    }
  }
}
